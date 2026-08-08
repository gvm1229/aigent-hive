//! Optional, outbound-only Discord notification for new usage-guard halts.
//!
//! The configured value is an environment-variable name. Webhook secrets stay
//! in the process environment and are never persisted, emitted, or added to a
//! diagnostic report. Delivery is best effort: guard enforcement remains
//! fail-closed even when Discord is unavailable.

use super::{emit_action_result, ActionResult};
use serde::Serialize;
use serde_json::json;
use std::env;
use std::process::ExitCode;
use std::time::Duration;

const DISCORD_TIMEOUT: Duration = Duration::from_secs(5);
const DELIVERY_ATTEMPTS: usize = 2;
const DISCORD_USAGE: &str = "\
Inspect the host-owned Discord inbound continuation boundary.

USAGE:
    hive discord inbound --host codex|claude|antigravity --output json
    hive discord test --webhook-env <ENVIRONMENT_NAME> [--language en|ko] [--fields <field,...>] --output json

Discord notification delivery remains outbound-only. Claude inbound handling is
delegated to the official Claude Discord Channel plugin. Codex continuation is
unsupported until an official compatible capability is verified.
";

/// Non-sensitive halt fields allowed in an outbound notification.
pub(crate) struct UsageHaltNotification<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) host_scope: &'a str,
    pub(crate) selected_window: &'a str,
    pub(crate) remaining_percent: Option<f64>,
    pub(crate) measured_at: u64,
    pub(crate) evidence_digest: &'a str,
    pub(crate) interface_language: &'a str,
    pub(crate) message_fields: &'a [String],
}

pub(crate) fn default_message_fields() -> Vec<String> {
    [
        "remaining-usage",
        "project",
        "request",
        "progress",
        "host",
        "resume",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

pub(crate) fn valid_message_fields(fields: &[String]) -> bool {
    !fields.is_empty()
        && fields.len() <= 8
        && fields.iter().all(|field| {
            matches!(
                field.as_str(),
                "remaining-usage"
                    | "project"
                    | "request"
                    | "progress"
                    | "host"
                    | "resume"
                    | "measured-at"
                    | "evidence"
            )
        })
        && fields
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == fields.len()
}

/// Opaque delivery outcome intentionally excluding URL and environment values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NotificationOutcome {
    Disabled,
    MissingWebhookEnvironment,
    InvalidWebhookUrl,
    Sent,
    DeliveryFailed,
}

impl NotificationOutcome {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::MissingWebhookEnvironment => "missing-webhook-environment",
            Self::InvalidWebhookUrl => "invalid-webhook-url",
            Self::Sent => "sent",
            Self::DeliveryFailed => "delivery-failed",
        }
    }
}

/// Return the explicit host boundary for Discord-originated continuation.
pub(crate) fn run(arguments: &[String]) -> ExitCode {
    if arguments.is_empty() || arguments.iter().any(|argument| argument == "--help") {
        print!("{DISCORD_USAGE}");
        return ExitCode::SUCCESS;
    }
    let result = match arguments.first().map(String::as_str) {
        Some("inbound") => parse_inbound(&arguments[1..]).map(inbound_result),
        Some("test") => parse_test(&arguments[1..]).map(test_result),
        Some(action) => Err(format!("unknown Discord action: {action}")),
        None => unreachable!("empty arguments returned above"),
    };
    let result = result.unwrap_or_else(|message| ActionResult {
        schema_version: 1,
        action: "InspectDiscordInbound",
        status: "error",
        exit_code: 2,
        code: "hive.invalid-input",
        message,
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    });
    emit_action_result(&result)
}

struct TestArguments {
    webhook_env: String,
    language: String,
    fields: Vec<String>,
}

fn parse_test(arguments: &[String]) -> Result<TestArguments, String> {
    let mut webhook_env = None;
    let mut language = "en".to_owned();
    let mut fields = default_message_fields();
    let mut output_json = false;
    let mut index = 0;
    while index < arguments.len() {
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| "Discord test option requires a value".to_owned())?;
        match arguments[index].as_str() {
            "--webhook-env" => webhook_env = Some(value.clone()),
            "--language" if matches!(value.as_str(), "en" | "ko") => language = value.clone(),
            "--fields" => fields = value.split(',').map(str::to_owned).collect(),
            "--output" if value == "json" => output_json = true,
            _ => {
                return Err(
                    "Discord test requires --webhook-env <ENVIRONMENT_NAME> [--language en|ko] [--fields <field,...>] --output json".to_owned(),
                )
            }
        }
        index += 2;
    }
    let webhook_env = webhook_env.ok_or_else(|| {
        "Discord test requires --webhook-env <ENVIRONMENT_NAME> [--language en|ko] [--fields <field,...>] --output json".to_owned()
    })?;
    if !output_json || !valid_environment_name(&webhook_env) || !valid_message_fields(&fields) {
        return Err(
            "Discord test requires a valid environment name, language, non-empty unique fields, and --output json".to_owned(),
        );
    }
    Ok(TestArguments {
        webhook_env,
        language,
        fields,
    })
}

fn test_result(arguments: TestArguments) -> ActionResult {
    let outcome = match env::var(&arguments.webhook_env) {
        Ok(url) => {
            let payload = payload_for(
                &UsageHaltNotification {
                    project_name: "example-project",
                    host_scope: "codex",
                    selected_window: "session",
                    remaining_percent: Some(20.0),
                    measured_at: 0,
                    evidence_digest: "sha256:test-notification",
                    interface_language: &arguments.language,
                    message_fields: &arguments.fields,
                },
                true,
            );
            notify_payload_with_url(&url, &payload, deliver_https)
        }
        Err(_) => NotificationOutcome::MissingWebhookEnvironment,
    };
    let (status, exit_code, code, message) = match outcome {
        NotificationOutcome::Sent => (
            "success",
            0,
            "hive.discord-test-sent",
            "Discord connection test sent",
        ),
        NotificationOutcome::MissingWebhookEnvironment => (
            "verification-failed",
            5,
            "hive.discord-test-missing-webhook",
            "Discord connection test could not read a valid webhook from the configured environment",
        ),
        NotificationOutcome::InvalidWebhookUrl | NotificationOutcome::DeliveryFailed => (
            "verification-failed",
            5,
            "hive.discord-test-delivery-failed",
            "Discord connection test could not be delivered",
        ),
        NotificationOutcome::Disabled => unreachable!("test never disables delivery"),
    };
    ActionResult {
        schema_version: 1,
        action: "TestDiscordWebhook",
        status,
        exit_code,
        code,
        message: message.to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: Some(json!({ "outcome": outcome.as_str() })),
    }
}

fn parse_inbound(arguments: &[String]) -> Result<&str, String> {
    if arguments.len() != 4
        || arguments.first().map(String::as_str) != Some("--host")
        || arguments.get(2).map(String::as_str) != Some("--output")
        || arguments.get(3).map(String::as_str) != Some("json")
    {
        return Err("Discord inbound requires --host <host> --output json".to_owned());
    }
    match arguments.get(1).map(String::as_str) {
        Some("codex" | "claude" | "antigravity") => Ok(arguments[1].as_str()),
        _ => Err("Discord inbound host must be codex, claude, or antigravity".to_owned()),
    }
}

fn inbound_result(host: &str) -> ActionResult {
    let (status, exit_code, code, message, owner) = match host {
        "claude" => (
            "success",
            0,
            "hive.discord-inbound-delegated",
            "Claude Discord inbound continuation is owned by the official Claude Discord Channel plugin",
            "claude-official-discord-channel",
        ),
        "codex" => (
            "unsupported",
            4,
            "hive.discord-inbound-unsupported",
            "Codex Discord inbound session continuation is unsupported until an official compatible capability is verified",
            "none",
        ),
        "antigravity" => (
            "unsupported",
            4,
            "hive.discord-inbound-unsupported",
            "Antigravity Discord inbound session continuation is unsupported",
            "none",
        ),
        _ => unreachable!("parse_inbound validates the host"),
    };
    ActionResult {
        schema_version: 1,
        action: "InspectDiscordInbound",
        status,
        exit_code,
        code,
        message: message.to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: Some(json!({
            "direction": "inbound",
            "owner": owner,
            "outbound_notification_only": true,
        })),
    }
}

#[derive(Serialize)]
struct DiscordPayload {
    content: String,
    allowed_mentions: AllowedMentions,
}

#[derive(Serialize)]
struct AllowedMentions {
    parse: Vec<String>,
}

/// Deliver one fresh halt notification if the optional integration is enabled.
pub(crate) fn notify_usage_halt(
    enabled: bool,
    webhook_url_env: Option<&str>,
    halt: &UsageHaltNotification<'_>,
) -> NotificationOutcome {
    let Some(environment_name) = enabled.then_some(webhook_url_env).flatten() else {
        return if enabled {
            NotificationOutcome::MissingWebhookEnvironment
        } else {
            NotificationOutcome::Disabled
        };
    };
    let Ok(url) = env::var(environment_name) else {
        return NotificationOutcome::MissingWebhookEnvironment;
    };
    notify_with_url(&url, halt, deliver_https)
}

fn notify_with_url<F>(
    url: &str,
    halt: &UsageHaltNotification<'_>,
    deliver: F,
) -> NotificationOutcome
where
    F: FnMut(&str, &[u8]) -> Result<(), ()>,
{
    notify_payload_with_url(url, &payload_for(halt, false), deliver)
}

fn notify_payload_with_url<F>(
    url: &str,
    payload: &DiscordPayload,
    mut deliver: F,
) -> NotificationOutcome
where
    F: FnMut(&str, &[u8]) -> Result<(), ()>,
{
    if !valid_webhook_url(url) {
        return NotificationOutcome::InvalidWebhookUrl;
    }
    let Ok(payload) = serde_json::to_vec(payload) else {
        return NotificationOutcome::DeliveryFailed;
    };
    for _ in 0..DELIVERY_ATTEMPTS {
        if deliver(url, &payload).is_ok() {
            return NotificationOutcome::Sent;
        }
    }
    NotificationOutcome::DeliveryFailed
}

fn valid_environment_name(value: &str) -> bool {
    let mut characters = value.chars();
    matches!(characters.next(), Some('A'..='Z' | '_'))
        && value.chars().count() <= 128
        && characters.all(|character| matches!(character, 'A'..='Z' | '0'..='9' | '_'))
}

fn payload_for(halt: &UsageHaltNotification<'_>, test: bool) -> DiscordPayload {
    let korean = halt.interface_language == "ko";
    let mut lines = Vec::<String>::new();
    if test {
        lines.push(
            if korean {
                "이 알림은 시험 메시지입니다. 이 메시지 형식은 자유롭게 변경을 요청할 수 있습니다."
            } else {
                "This is a test message. You can freely ask to change this message's format."
            }
            .to_owned(),
        );
    }
    lines.push(
        if korean {
            "Aigent Hive 사용량 보호가 작업을 중단했습니다."
        } else {
            "Aigent Hive usage guard stopped a workflow."
        }
        .to_owned(),
    );
    for field in halt.message_fields {
        let line = match field.as_str() {
            "remaining-usage" => format!(
                "{}: {}",
                if korean {
                    "남은 사용량"
                } else {
                    "remaining usage"
                },
                display_remaining_usage(halt.remaining_percent, korean)
            ),
            "project" => format!(
                "{}: {}",
                if korean { "프로젝트" } else { "project" },
                display_project_name(halt.project_name)
            ),
            "request" if korean => "요청: 공유하지 않음(요청 내용은 이 컴퓨터에 유지)".to_owned(),
            "request" => "request: not shared (request content stays local)".to_owned(),
            "progress" if korean => {
                "진행 상태: 알 수 없음(확인 가능한 작업 진행 정보 없음)".to_owned()
            }
            "progress" => "progress: unknown (no canonical run progress is available)".to_owned(),
            "host" => format!(
                "{}: {} · {}",
                if korean { "호스트" } else { "host" },
                halt.host_scope,
                halt.selected_window
            ),
            "resume" if korean => {
                "계속하기: 이 프로젝트로 돌아가 Hive에게 계속 진행을 요청".to_owned()
            }
            "resume" => "resume: return to this project and ask Hive to continue".to_owned(),
            "measured-at" => format!(
                "{}: {}",
                if korean {
                    "측정 시각"
                } else {
                    "measured at"
                },
                halt.measured_at
            ),
            "evidence" => format!(
                "{}: {}",
                if korean {
                    "검증 참조"
                } else {
                    "evidence reference"
                },
                halt.evidence_digest
            ),
            _ => continue,
        };
        lines.push(line);
    }
    DiscordPayload {
        content: lines.join("\n"),
        allowed_mentions: AllowedMentions { parse: Vec::new() },
    }
}

fn display_remaining_usage(remaining_percent: Option<f64>, korean: bool) -> String {
    remaining_percent
        .filter(|value| value.is_finite() && (0.0..=100.0).contains(value))
        .map(|value| format!("{value:.2}%"))
        .unwrap_or_else(|| {
            if korean {
                "알 수 없음".to_owned()
            } else {
                "unknown".to_owned()
            }
        })
}

fn display_project_name(value: &str) -> String {
    let allowed = !value.is_empty()
        && value.len() <= 80
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, ' ' | '.' | '_' | '-')
        });
    if allowed {
        value.to_owned()
    } else {
        format!(
            "project-{}",
            &hive_core::sha256_digest(value.as_bytes())[7..19]
        )
    }
}

fn valid_webhook_url(url: &str) -> bool {
    let prefix = [
        "https://discord.com/api/webhooks/",
        "https://discordapp.com/api/webhooks/",
    ]
    .into_iter()
    .find(|prefix| url.starts_with(prefix));
    let Some(prefix) = prefix else {
        return false;
    };
    if url.len() > 2048 || url.chars().any(char::is_control) || url.contains(['?', '#', ' ']) {
        return false;
    }
    let segments = url[prefix.len()..].split('/').collect::<Vec<_>>();
    segments.len() == 2
        && segments
            .iter()
            .all(|segment| !segment.is_empty() && segment.len() <= 512)
}

fn deliver_https(url: &str, payload: &[u8]) -> Result<(), ()> {
    let config = ureq::Agent::config_builder()
        .https_only(true)
        .timeout_global(Some(DISCORD_TIMEOUT))
        .user_agent(concat!("aigent-hive/", env!("CARGO_PKG_VERSION")))
        .build();
    let agent: ureq::Agent = config.into();
    agent
        .post(url)
        .header("content-type", "application/json")
        .send(payload)
        .map(|_| ())
        .map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::{
        default_message_fields, display_project_name, inbound_result, notify_with_url, parse_test,
        payload_for, valid_webhook_url, NotificationOutcome, UsageHaltNotification,
    };

    fn notification<'a>(fields: &'a [String]) -> UsageHaltNotification<'a> {
        UsageHaltNotification {
            project_name: "aigent-hive",
            host_scope: "codex",
            selected_window: "weekly",
            remaining_percent: Some(18.5),
            measured_at: 1_700_000_000,
            evidence_digest: "sha256:allowed-evidence",
            interface_language: "en",
            message_fields: fields,
        }
    }

    #[test]
    fn payload_excludes_session_prompt_and_credentials() {
        let fields = default_message_fields();
        let payload =
            serde_json::to_string(&payload_for(&notification(&fields), false)).expect("payload");

        assert!(payload.contains("allowed_mentions"));
        assert!(!payload.contains("session"));
        assert!(!payload.contains("prompt"));
        assert!(!payload.contains("token"));
        assert!(!payload.contains("webhook"));
        assert!(payload.contains("project: aigent-hive"));
        assert!(payload.contains("progress: unknown"));
        assert!(payload.contains("remaining usage: 18.50%"));
    }

    #[test]
    fn test_payload_matches_the_actual_payload_except_for_the_disclaimer() {
        let fields = default_message_fields();
        let actual = payload_for(&notification(&fields), false).content;
        let test = payload_for(&notification(&fields), true).content;

        assert!(test.starts_with("This is a test message."));
        assert_eq!(
            test.lines().skip(1).collect::<Vec<_>>(),
            actual.lines().collect::<Vec<_>>()
        );
    }

    #[test]
    fn korean_payload_uses_korean_labels_without_english_label_mixup() {
        let fields = vec!["remaining-usage".to_owned(), "project".to_owned()];
        let mut notification = notification(&fields);
        notification.interface_language = "ko";
        let payload = payload_for(&notification, true).content;

        assert!(payload.starts_with("이 알림은 시험 메시지입니다."));
        assert!(payload.contains("남은 사용량: 18.50%"));
        assert!(payload.contains("프로젝트: aigent-hive"));
        assert!(!payload.contains("remaining usage:"));
        assert!(!payload.contains("project:"));
    }

    #[test]
    fn test_command_accepts_the_same_language_and_ordered_fields_as_the_alert() {
        let arguments = [
            "--webhook-env",
            "HIVE_DISCORD_WEBHOOK_URL",
            "--language",
            "ko",
            "--fields",
            "project,remaining-usage",
            "--output",
            "json",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        let parsed = parse_test(&arguments).expect("test command arguments");

        assert_eq!(parsed.language, "ko");
        assert_eq!(parsed.fields, ["project", "remaining-usage"]);
    }

    #[test]
    fn unsafe_project_name_is_replaced_with_a_stable_non_path_identifier() {
        assert!(display_project_name("../../secret").starts_with("project-"));
    }

    #[test]
    fn only_discord_https_webhook_paths_are_allowed() {
        assert!(valid_webhook_url(
            "https://discord.com/api/webhooks/1234567890/a-valid-token"
        ));
        assert!(!valid_webhook_url("http://discord.com/api/webhooks/1/2"));
        assert!(!valid_webhook_url("https://example.com/api/webhooks/1/2"));
        assert!(!valid_webhook_url(
            "https://discord.com/api/webhooks/1/2?wait=true"
        ));
    }

    #[test]
    fn delivery_retries_once_without_exposing_the_webhook() {
        let mut calls = 0;
        let outcome = notify_with_url(
            "https://discord.com/api/webhooks/1234567890/a-valid-token",
            &notification(&default_message_fields()),
            |_, _| {
                calls += 1;
                Err(())
            },
        );

        assert_eq!(outcome, NotificationOutcome::DeliveryFailed);
        assert_eq!(calls, 2);
    }

    #[test]
    fn codex_inbound_continuation_is_truthfully_unsupported() {
        let result = inbound_result("codex");

        assert_eq!(result.status, "unsupported");
        assert_eq!(result.code, "hive.discord-inbound-unsupported");
        assert_eq!(result.exit_code, 4);
    }
}
