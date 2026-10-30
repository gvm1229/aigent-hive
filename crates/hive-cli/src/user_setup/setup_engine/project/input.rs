//! Context-dependent checks run at the affected question, before the final preview.
use hive_core::setup::Question;
use hive_render::GlobalProjectPreferences;
use serde_json::{json, Value};

pub(super) fn normalize(question: &Question, value: &mut Value) {
    if matches!(
        question.id.as_str(),
        "project-knowledge-promotion" | "project-knowledge-confidential"
    ) {
        if let Some(values) = value.as_array_mut() {
            values.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
        }
    }
}

pub(super) fn fallback(question: &Question, global: &GlobalProjectPreferences) -> Option<Value> {
    match question.id.as_str() {
        "project-threshold" => Some(json!(global.usage_stop_remaining_percent)),
        "project-wiki" if !global.wiki_enabled => Some(json!(false)),
        _ => None,
    }
}

pub(super) fn error(
    question: &Question,
    answers: &Value,
    value: &Value,
    global: &GlobalProjectPreferences,
) -> Option<(&'static str, &'static str)> {
    match question.id.as_str() {
        "project-threshold" if global.usage_guard_enabled && value.as_u64().is_some_and(|value|value<u64::from(global.usage_stop_remaining_percent)) =>
            Some(("보호가 켜져 있어 전역 기준보다 낮출 수 없습니다.","Enforcement is enabled, so the project threshold cannot be below the global threshold.")),
        "project-wiki" if value==true && !global.wiki_enabled =>
            Some(("전역 문서 저장을 먼저 켜야 프로젝트에서도 사용할 수 있습니다.","Enable global knowledge storage before enabling it for this project.")),
        "project-knowledge-include"|"project-knowledge-exclude" if value.as_array().is_some_and(|paths|paths.iter().any(|path|path.as_str().is_some_and(|path|
            path.is_empty()||path.starts_with(['/', '\\'])||path.as_bytes().get(1)==Some(&b':')||path.split(['/', '\\']).any(|part|part=="..")))) =>
            Some(("프로젝트 안의 상대 경로를 사용해 주세요. 상위 폴더 이동은 허용하지 않습니다.","Use paths relative to this project, without parent-directory traversal.")),
        "project-knowledge-confidential" if overlaps(value,&answers["root_knowledge_promotion_categories"]) =>
            Some(("공유 대상으로 고른 종류를 동시에 비공개로 지정할 수 없습니다.","A category selected for sharing cannot also be confidential.")),
        _=>None,
    }
}

fn overlaps(left: &Value, right: &Value) -> bool {
    left.as_array().is_some_and(|left| {
        right
            .as_array()
            .is_some_and(|right| left.iter().any(|value| right.contains(value)))
    })
}
