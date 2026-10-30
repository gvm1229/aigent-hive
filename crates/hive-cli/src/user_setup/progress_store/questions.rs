//! Reconcile unconfirmed drafts without turning a changed catalog into new consent.
use super::{ProgressPhase, ProgressSnapshot, ProgressStore, SetupError, QUESTION_REVISION};
use hive_core::{
    preferences::PreferenceScope,
    setup::{InputKind, Question},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::user_setup) struct QuestionBinding {
    pub revision: u32,
    pub key: String,
    pub input: InputKind,
    pub section: String,
}

impl From<&Question> for QuestionBinding {
    fn from(question: &Question) -> Self {
        Self {
            revision: question.revision,
            key: question.key.clone(),
            input: question.input,
            section: question.section.clone(),
        }
    }
}

pub(super) fn capture(
    scope: PreferenceScope,
) -> Result<BTreeMap<String, QuestionBinding>, SetupError> {
    Ok(crate::user_setup::setup_engine::catalog()?
        .questions
        .iter()
        .filter(|question| question.scope == scope)
        .map(|question| (question.id.clone(), QuestionBinding::from(question)))
        .collect())
}

pub(super) fn validate_bindings(
    bindings: &BTreeMap<String, QuestionBinding>,
) -> Result<(), SetupError> {
    if bindings.len() > 128 {
        return Err(SetupError::Verification(
            "too many saved setup questions".into(),
        ));
    }
    for (id, binding) in bindings {
        crate::user_setup::validate_progress_step(id)?;
        crate::user_setup::validate_progress_step(&binding.section)?;
        if binding.revision == 0
            || !binding.key.starts_with('/')
            || binding.key.len() > 256
            || binding.key.chars().any(char::is_control)
        {
            return Err(SetupError::Verification(
                "invalid saved setup question definition".into(),
            ));
        }
    }
    Ok(())
}

impl ProgressStore {
    pub(in crate::user_setup) fn refresh_draft_questions(
        &self,
        snapshot: &ProgressSnapshot,
    ) -> Result<Option<ProgressSnapshot>, SetupError> {
        let Some(prior) = &snapshot.progress else {
            return Ok(None);
        };
        if self.resume_ready(prior)
            || !matches!(
                prior.phase,
                ProgressPhase::AwaitingAnswers | ProgressPhase::Previewed
            )
            || prior.base_config_digest != self.base_config_digest
            || prior.base_config_mode != self.base_config_mode
            || prior.question_revision != QUESTION_REVISION
        {
            return Ok(None);
        }
        let current = capture(self.binding.scope)?;
        if prior
            .answered_questions
            .keys()
            .any(|id| !current.contains_key(id))
        {
            return Err(SetupError::Conflict(
                "a saved question is no longer supported; answers were preserved".into(),
            ));
        }
        let mut progress = prior.clone();
        // Keep the prior definitions for answered fields. The terminal rechecks only those
        // whose meaning revision, key, input type, or current value no longer matches.
        for (id, binding) in current {
            if !progress.answered_questions.contains_key(&id) {
                progress.question_bindings.insert(id, binding);
            }
        }
        progress.catalog_digest.clone_from(&self.catalog_digest);
        progress.phase = ProgressPhase::AwaitingAnswers;
        self.save(snapshot, &progress).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hive_core::sha256_digest;
    use serde_json::json;
    use std::fs;

    fn stale(store: &ProgressStore, phase: ProgressPhase) -> ProgressSnapshot {
        let mut progress = store
            .new_progress(
                "user-contexts".into(),
                json!({
                    "interface_language":"ko", "profile":{"contexts":["web-developer"]}
                }),
            )
            .unwrap();
        progress
            .answered_questions
            .insert("user-language".into(), 1);
        progress
            .answered_questions
            .insert("user-contexts".into(), 2);
        progress
            .question_bindings
            .get_mut("user-contexts")
            .unwrap()
            .revision = 2;
        progress.phase = phase;
        store.save(&store.load().unwrap(), &progress).unwrap();
        progress.catalog_digest = sha256_digest(b"prior catalog");
        fs::write(
            store.root_path.join(&store.relative),
            serde_yaml::to_string(&progress).unwrap(),
        )
        .unwrap();
        store.load().unwrap()
    }

    #[test]
    fn changed_catalog_keeps_values_and_prior_definitions_for_selective_review() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let prior = stale(&store, ProgressPhase::Previewed);
        let refreshed = store.refresh_draft_questions(&prior).unwrap().unwrap();
        let before = prior.progress.as_ref().unwrap();
        let after = refreshed.progress.as_ref().unwrap();
        assert_eq!(after.answers, before.answers);
        assert_eq!(after.operation_id, before.operation_id);
        assert_eq!(after.question_bindings["user-contexts"].revision, 2);
        assert_eq!(after.answered_questions, before.answered_questions);
        assert_eq!(after.phase, ProgressPhase::AwaitingAnswers);
        assert!(store.resume_ready(after));
        assert!(store.refresh_draft_questions(&refreshed).unwrap().is_none());
    }

    #[test]
    fn applying_intents_and_changed_settings_are_not_reinterpreted() {
        for phase in [ProgressPhase::Applying, ProgressPhase::RecoverableFailure] {
            let root = tempfile::tempdir().unwrap();
            let store = ProgressStore::open(root.path(), None).unwrap();
            let prior = stale(&store, phase);
            let bytes = fs::read(root.path().join(&store.relative)).unwrap();
            assert!(store.refresh_draft_questions(&prior).unwrap().is_none());
            assert_eq!(fs::read(root.path().join(&store.relative)).unwrap(), bytes);
        }
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let prior = stale(&store, ProgressPhase::AwaitingAnswers);
        let bytes = fs::read(root.path().join(&store.relative)).unwrap();
        fs::write(
            root.path().join(super::super::USER_SETUP_RELATIVE),
            "externally edited settings",
        )
        .unwrap();
        let changed = ProgressStore::open(root.path(), None).unwrap();
        assert!(changed.refresh_draft_questions(&prior).unwrap().is_none());
        assert_eq!(fs::read(root.path().join(&store.relative)).unwrap(), bytes);
    }

    #[test]
    fn removed_questions_and_changed_protocol_preserve_the_original_record() {
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut prior = stale(&store, ProgressPhase::AwaitingAnswers);
        prior
            .progress
            .as_mut()
            .unwrap()
            .answered_questions
            .insert("retired-question".into(), 1);
        assert!(store.refresh_draft_questions(&prior).is_err());
        let mut prior = store.load().unwrap();
        prior.progress.as_mut().unwrap().question_revision += 1;
        assert!(store.refresh_draft_questions(&prior).unwrap().is_none());
        assert!(!store.resume_ready(store.load().unwrap().progress.as_ref().unwrap()));
    }
}
