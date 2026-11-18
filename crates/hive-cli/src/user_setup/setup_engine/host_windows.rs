//! Per-host usage window questions: catalog templates repeated once for each selected app.
//!
//! A template key contains `{host}`. Expansion keeps the catalog order and repeats the whole
//! template group for each selected host, in the stored host order.
use super::super::SetupError;
use hive_core::setup::Question;
use serde_json::{json, Value};

const PLACEHOLDER: &str = "{host}";
const LIMIT: &str = "usage-host-five-hour-limit";
const GUARD: &str = "usage-host-guard";
const FIVE_HOUR_THRESHOLD: &str = "usage-host-five-hour-threshold";
const WEEKLY_THRESHOLD: &str = "usage-host-weekly-threshold";
const HOSTS: [&str; 3] = ["codex", "claude", "antigravity"];

fn is_template(question: &Question) -> bool {
    question.key.contains(PLACEHOLDER)
}

/// Display name used in questions and titles.
pub(super) fn host_label(host: &str) -> &str {
    match host {
        "codex" => "Codex",
        "claude" => "Claude Code",
        "antigravity" => "Antigravity",
        other => other,
    }
}

fn instantiate(template: &Question, host: &str) -> Question {
    let mut question = template.clone();
    question.id = format!("{}-{host}", template.id);
    question.key = template.key.replace(PLACEHOLDER, host);
    for condition in &mut question.when {
        condition.key = condition.key.replace(PLACEHOLDER, host);
    }
    let label = host_label(host);
    question.prompt.ko = template.prompt.ko.replace(PLACEHOLDER, label);
    question.prompt.en = template.prompt.en.replace(PLACEHOLDER, label);
    question
}

/// Selected hosts in their stored order.
pub(super) fn selected_hosts(answers: &Value) -> Vec<String> {
    answers["selected_hosts"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|host| HOSTS.contains(host))
        .map(str::to_owned)
        .collect()
}

/// Replace the template group with one concrete group per selected host.
pub(super) fn expand(questions: &[Question], answers: &Value) -> Vec<Question> {
    let templates: Vec<&Question> = questions.iter().filter(|q| is_template(q)).collect();
    let hosts = selected_hosts(answers);
    let mut expanded = Vec::with_capacity(questions.len() + templates.len() * hosts.len());
    let mut inserted = false;
    for question in questions {
        if !is_template(question) {
            expanded.push(question.clone());
        } else if !inserted {
            inserted = true;
            for host in &hosts {
                expanded.extend(templates.iter().map(|template| instantiate(template, host)));
            }
        }
    }
    expanded
}

/// The template id and host of a concrete question.
pub(super) fn instance(id: &str) -> Option<(&str, &str)> {
    HOSTS.iter().find_map(|host| {
        id.strip_suffix(host)
            .and_then(|prefix| prefix.strip_suffix('-'))
            .filter(|template| {
                [LIMIT, GUARD, FIVE_HOUR_THRESHOLD, WEEKLY_THRESHOLD].contains(template)
            })
            .map(|template| (template, *host))
    })
}

/// Create the host's answer object so that a concrete key has a parent.
pub(super) fn prepare(answers: &mut Value, question: &Question) {
    if let Some((_, host)) = instance(&question.id) {
        let windows = &mut answers["usage_guard"]["host_windows"];
        if !windows.is_object() {
            *windows = json!({});
        }
        if !windows[host].is_object() {
            windows[host] = json!({});
        }
    }
}

/// Keep one host's stored answers consistent after a limit or guard answer.
/// Returns the keys whose stale answers were removed.
pub(super) fn settle(answers: &mut Value, question: &Question) -> Result<Vec<String>, SetupError> {
    let Some((template, host)) = instance(&question.id) else {
        return Ok(Vec::new());
    };
    let entry = answers["usage_guard"]["host_windows"][host]
        .as_object_mut()
        .ok_or_else(|| SetupError::Internal("missing host usage answers".into()))?;
    if template == LIMIT && entry.get("five_hour_limit") == Some(&Value::Bool(false)) {
        entry.insert("guard".into(), json!("weekly"));
    }
    if !matches!(template, LIMIT | GUARD) {
        return Ok(Vec::new());
    }
    let guard = entry
        .get("guard")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let mut removed = Vec::new();
    for (field, guarded) in [
        (
            "five_hour_stop_remaining_percent",
            matches!(guard, "five-hour" | "both"),
        ),
        (
            "weekly_stop_remaining_percent",
            matches!(guard, "weekly" | "both"),
        ),
    ] {
        if !guarded && entry.remove(field).is_some() {
            removed.push(format!("/usage_guard/host_windows/{host}/{field}"));
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hive_core::preferences::PreferenceScope;

    fn usage_questions(answers: &Value) -> Vec<Question> {
        expand(&super::super::catalog().unwrap().questions, answers)
            .into_iter()
            .filter(|q| q.scope == PreferenceScope::User && q.applicable(answers))
            .filter(|q| instance(&q.id).is_some())
            .collect()
    }

    #[test]
    fn each_selected_host_gets_one_question_group_in_host_order() {
        for hosts in [json!(["claude"]), json!(["claude", "codex", "antigravity"])] {
            let count = hosts.as_array().unwrap().len();
            let answers = json!({"selected_hosts":hosts,
                "usage_guard":{"enforcement_enabled":true}});
            let limits: Vec<String> = usage_questions(&answers)
                .into_iter()
                .filter(|q| instance(&q.id).is_some_and(|(t, _)| t == LIMIT))
                .map(|q| q.id)
                .collect();
            assert_eq!(limits.len(), count);
            let expected: Vec<String> = hosts
                .as_array()
                .unwrap()
                .iter()
                .map(|h| format!("{LIMIT}-{}", h.as_str().unwrap()))
                .collect();
            assert_eq!(limits, expected);
        }
        let off = json!({"selected_hosts":["claude"],"usage_guard":{"enforcement_enabled":false}});
        assert!(usage_questions(&off).is_empty());
    }

    #[test]
    fn answers_select_only_the_threshold_questions_they_need() {
        let ids = |windows: Value| {
            let answers = json!({"selected_hosts":["claude"],
                "usage_guard":{"enforcement_enabled":true,"host_windows":{"claude":windows}}});
            usage_questions(&answers)
                .into_iter()
                .map(|q| instance(&q.id).unwrap().0.to_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            ids(json!({"five_hour_limit":false,"guard":"weekly"})),
            [LIMIT, WEEKLY_THRESHOLD]
        );
        assert_eq!(
            ids(json!({"five_hour_limit":true,"guard":"five-hour"})),
            [LIMIT, GUARD, FIVE_HOUR_THRESHOLD]
        );
        assert_eq!(
            ids(json!({"five_hour_limit":true,"guard":"weekly"})),
            [LIMIT, GUARD, WEEKLY_THRESHOLD]
        );
        assert_eq!(
            ids(json!({"five_hour_limit":true,"guard":"both"})),
            [LIMIT, GUARD, FIVE_HOUR_THRESHOLD, WEEKLY_THRESHOLD]
        );
    }

    #[test]
    fn settling_a_choice_removes_thresholds_for_unguarded_windows() {
        let questions = expand(
            &super::super::catalog().unwrap().questions,
            &json!({"selected_hosts":["claude"]}),
        );
        let find = |id: &str| questions.iter().find(|q| q.id == id).unwrap().clone();
        let mut answers = json!({"usage_guard":{"host_windows":{"claude":{"five_hour_limit":false,
            "five_hour_stop_remaining_percent":5,"weekly_stop_remaining_percent":7}}}});
        let removed = settle(&mut answers, &find("usage-host-five-hour-limit-claude")).unwrap();
        assert_eq!(
            removed,
            ["/usage_guard/host_windows/claude/five_hour_stop_remaining_percent"]
        );
        assert_eq!(
            answers["usage_guard"]["host_windows"]["claude"]["guard"],
            "weekly"
        );
        assert_eq!(instance("usage-host-guard-claude"), Some((GUARD, "claude")));
        assert_eq!(instance("usage-threshold"), None);
        assert!(find("usage-host-guard-claude")
            .prompt
            .en
            .contains("Claude Code"));
    }
}
