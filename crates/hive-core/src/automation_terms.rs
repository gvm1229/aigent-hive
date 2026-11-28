//! Permanent refusal of Antigravity and the retired Gemini CLI as automated Hive workers.
//!
//! Google's Antigravity terms prohibit third-party software from accessing Antigravity, and
//! accounts can be suspended for it. Hive therefore never uses `agy`, an Antigravity
//! subscription login or the retired Gemini CLI as a worker, subagent, delegate, reviewer,
//! critic, judge, dispatch or continuation target, or automatic re-invocation target.
//! Human-interactive Antigravity, Skill projection into its folders, `agy --version`,
//! `agy plugin` install management and Antigravity's own usage callback stay supported.

/// Shared English refusal reason.
pub const ANTIGRAVITY_AUTOMATION_REFUSAL: &str = "Antigravity/Gemini cannot be an automated Hive worker: Google's Antigravity terms prohibit third-party tools accessing it (account suspension risk); use it directly yourself.";

/// Shared Korean refusal reason for Korean human output.
pub const ANTIGRAVITY_AUTOMATION_REFUSAL_KO: &str = "Antigravity·Gemini는 Hive 자동 작업자가 될 수 없습니다: Google Antigravity 약관이 제3자 도구의 접근을 금지합니다(계정 정지 위험). 직접 사용하세요.";

/// Return whether one host or provider name names Antigravity or the retired Gemini CLI.
#[must_use]
pub fn is_terms_restricted(name: &str) -> bool {
    matches!(
        name.trim().to_ascii_lowercase().as_str(),
        "gemini" | "antigravity" | "agy"
    )
}

/// Refuse one host or provider name used as an automated target.
///
/// # Errors
///
/// Returns the shared refusal reason when `name` is Antigravity or Gemini.
pub fn refuse_automated(name: &str) -> Result<(), String> {
    if is_terms_restricted(name) {
        Err(ANTIGRAVITY_AUTOMATION_REFUSAL.to_owned())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn antigravity_and_gemini_are_refused_and_others_are_not() {
        for name in ["gemini", "antigravity", "agy", " Antigravity "] {
            assert!(is_terms_restricted(name));
            assert_eq!(
                refuse_automated(name).unwrap_err(),
                ANTIGRAVITY_AUTOMATION_REFUSAL
            );
        }
        for name in ["codex", "claude", ""] {
            assert!(refuse_automated(name).is_ok());
        }
    }
}
