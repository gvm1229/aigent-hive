//! Save a search preference without installing or enabling a search runtime.
use super::{
    catalog,
    interactive::{persist, Conversation},
    terminal::Terminal,
};
use crate::user_setup::{
    self,
    progress_store::{optional_digest, ProgressStore, SetupProgress},
    SetupError, UserFeatureAnswers, VectorFeatureAnswer,
};
use cap_std::fs::Dir;
use hive_core::{
    preferences::{PreferenceDecision, PreferenceInput, PreferenceScope},
    setup::AnswerInput,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{BufRead, Write};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::user_setup) struct Selection {
    before_digest: String,
    desired: UserFeatureAnswers,
}

pub(super) fn collect<R: BufRead, W: Write>(
    root: &Dir,
    store: &ProgressStore,
    progress: &mut Conversation,
    terminal: &mut Terminal<R, W>,
) -> Result<bool, SetupError> {
    let explicit = progress.selected_section.as_deref() == Some("vector");
    if progress
        .selected_section
        .as_deref()
        .is_some_and(|section| section != "vector")
    {
        return Ok(true);
    }
    let question = catalog()?
        .questions
        .into_iter()
        .find(|question| question.id == "user-vector-search")
        .expect("vector question");
    if progress.vector_choice.as_ref().is_some_and(|selection| {
        selection
            .desired
            .decision
            .as_ref()
            .is_none_or(|decision| decision.question_revision != Some(question.revision))
    }) {
        progress.vector_choice = None;
        progress.answered_questions.remove(&question.id);
    }
    if let Some(selection) = &progress.vector_choice {
        let (bytes, _) = user_setup::load_feature_answers(root)?;
        if optional_digest(bytes.as_deref()) != selection.before_digest {
            return Err(SetupError::Conflict(
                "semantic search preference changed during setup".into(),
            ));
        }
        return Ok(true);
    }
    let (bytes, mut desired) = user_setup::load_feature_answers(root)?;
    if !explicit && desired.vector_search.is_some() {
        return Ok(true);
    }
    progress.step.clone_from(&question.id);
    persist(store, progress)?;
    let current = desired
        .vector_search
        .map(|answer| json!(answer == VectorFeatureAnswer::Yes));
    let AnswerInput::Answered { value, origin } = terminal.ask(&question, current.as_ref())? else {
        return Ok(false);
    };
    let now = user_setup::feature_now_unix()?;
    let mut decision = PreferenceDecision::new(
        &value,
        origin,
        PreferenceInput::Terminal,
        PreferenceScope::User,
        env!("CARGO_PKG_VERSION"),
        Some(now),
    )
    .map_err(SetupError::Input)?;
    decision.question_id = Some(question.id.clone());
    decision.question_revision = Some(question.revision);
    desired.vector_search = Some(if value.as_bool().expect("boolean question") {
        VectorFeatureAnswer::Yes
    } else {
        VectorFeatureAnswer::No
    });
    desired
        .introduced_in
        .get_or_insert_with(|| env!("CARGO_PKG_VERSION").into());
    desired.answered_at_unix = Some(now);
    desired.question_claimed_at_unix = None;
    desired.decision = Some(decision);
    desired.setup_operation = Some(progress.operation_id.clone());
    progress.vector_choice = Some(Selection {
        before_digest: optional_digest(bytes.as_deref()),
        desired,
    });
    progress
        .answered_questions
        .insert(question.id.clone(), question.revision);
    progress
        .question_bindings
        .insert(question.id.clone(), (&question).into());
    persist(store, progress)?;
    Ok(true)
}

pub(super) fn display<R: BufRead, W: Write>(
    progress: &SetupProgress,
    terminal: &mut Terminal<R, W>,
) -> Result<(), SetupError> {
    if let Some(selection) = &progress.vector_choice {
        let enabled = selection.desired.vector_search == Some(VectorFeatureAnswer::Yes);
        terminal.say(
            if enabled {
                "의미 검색 희망: 예. 다운로드·활성화는 별도 동의 후 진행."
            } else {
                "의미 검색 희망: 아니요. 기존 정확 검색 유지."
            },
            if enabled {
                "Semantic search preference: yes. Downloads and activation need separate consent."
            } else {
                "Semantic search preference: no. Exact search remains available."
            },
        )?;
    }
    Ok(())
}

pub(super) fn apply(root: &Dir, progress: &SetupProgress) -> Result<(), SetupError> {
    let Some(selection) = &progress.vector_choice else {
        return Ok(());
    };
    let value = json!(selection.desired.vector_search == Some(VectorFeatureAnswer::Yes));
    if selection.desired.setup_operation.as_deref() != Some(&progress.operation_id)
        || selection.desired.vector_search.is_none()
        || selection.desired.decision.as_ref().is_none_or(|decision| {
            decision.question_id.as_deref() != Some("user-vector-search")
                || !decision.matches_declaration(&value).unwrap_or(false)
        })
    {
        return Err(SetupError::Conflict(
            "saved semantic search choice is invalid".into(),
        ));
    }
    let (bytes, current) = user_setup::load_feature_answers(root)?;
    if current == selection.desired {
        return Ok(());
    }
    if optional_digest(bytes.as_deref()) != selection.before_digest {
        return Err(SetupError::Conflict(
            "semantic search preference changed after confirmation; preserved".into(),
        ));
    }
    user_setup::save_feature_answers(root, bytes.as_deref(), &selection.desired)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_setup::InterfaceLanguage;
    use std::io::Cursor;

    fn conversation(store: &ProgressStore) -> Conversation {
        Conversation {
            record: store
                .new_progress(
                    "test".into(),
                    super::super::default_answers(Some(InterfaceLanguage::En)),
                )
                .unwrap(),
            snapshot: store.load().unwrap(),
            fresh: true,
        }
    }

    #[test]
    fn choice_waits_for_apply_preserves_provenance_and_replays_without_writes() {
        for (input, expected) in [
            (b"yes\n".as_slice(), VectorFeatureAnswer::Yes),
            (b"no\n".as_slice(), VectorFeatureAnswer::No),
        ] {
            let root = tempfile::tempdir().unwrap();
            let cap = crate::user_install::open_user_root_for_setup(root.path()).unwrap();
            let store = ProgressStore::open(root.path(), None).unwrap();
            let mut progress = conversation(&store);
            let mut terminal = Terminal {
                reader: Cursor::new(input),
                writer: Vec::new(),
                language: InterfaceLanguage::En,
            };
            assert!(collect(&cap, &store, &mut progress, &mut terminal).unwrap());
            assert!(!root
                .path()
                .join(user_setup::USER_FEATURE_ANSWERS_RELATIVE)
                .exists());
            display(&progress, &mut terminal).unwrap();
            assert!(String::from_utf8(terminal.writer)
                .unwrap()
                .contains("preference only"));
            apply(&cap, &progress).unwrap();
            let (first, saved) = user_setup::load_feature_answers(&cap).unwrap();
            assert_eq!(saved.vector_search, Some(expected));
            assert_eq!(
                saved.decision.unwrap().input_channel,
                PreferenceInput::Terminal
            );
            apply(&cap, &progress).unwrap();
            assert_eq!(user_setup::load_feature_answers(&cap).unwrap().0, first);
            assert!(!root.path().join(".hive/runtime").exists());
        }
    }

    #[test]
    fn eof_and_other_sections_leave_the_feature_answer_untouched() {
        let root = tempfile::tempdir().unwrap();
        let cap = crate::user_install::open_user_root_for_setup(root.path()).unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = conversation(&store);
        let mut terminal = Terminal {
            reader: Cursor::new(b"".as_slice()),
            writer: Vec::new(),
            language: InterfaceLanguage::Ko,
        };
        assert!(!collect(&cap, &store, &mut progress, &mut terminal).unwrap());
        assert!(progress.vector_choice.is_none());
        assert!(String::from_utf8(terminal.writer.clone())
            .unwrap()
            .contains("희망 여부만 저장"));
        progress.selected_section = Some("persona".into());
        assert!(collect(&cap, &store, &mut progress, &mut terminal).unwrap());
        assert!(user_setup::load_feature_answers(&cap).unwrap().0.is_none());
    }

    #[test]
    fn prior_answers_are_preserved_and_later_choices_block_stale_apply() {
        let root = tempfile::tempdir().unwrap();
        let cap = crate::user_install::open_user_root_for_setup(root.path()).unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = conversation(&store);
        let mut terminal = Terminal {
            reader: Cursor::new(b"yes\n"),
            writer: Vec::new(),
            language: InterfaceLanguage::En,
        };
        assert!(collect(&cap, &store, &mut progress, &mut terminal).unwrap());
        apply(&cap, &progress).unwrap();
        let prior = user_setup::load_feature_answers(&cap).unwrap().0.unwrap();
        let mut next = conversation(&store);
        let mut empty = Terminal {
            reader: Cursor::new(b"".as_slice()),
            writer: Vec::new(),
            language: InterfaceLanguage::En,
        };
        assert!(collect(&cap, &store, &mut next, &mut empty).unwrap());
        assert!(empty.writer.is_empty() && next.vector_choice.is_none());
        next.selected_section = Some("vector".into());
        empty.reader = Cursor::new(b"no\n".as_slice());
        assert!(collect(&cap, &store, &mut next, &mut empty).unwrap());
        let mut external = prior.clone();
        external.extend_from_slice(b"# outside edit\n");
        std::fs::write(
            root.path().join(user_setup::USER_FEATURE_ANSWERS_RELATIVE),
            &external,
        )
        .unwrap();
        assert!(apply(&cap, &next).is_err());
        assert_eq!(
            user_setup::load_feature_answers(&cap).unwrap().0.unwrap(),
            external
        );
    }
}
