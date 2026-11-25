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
    hive discord inbound --host codex|claude|antigravity [--output json]
    hive discord test --webhook-env <ENVIRONMENT_NAME> [--language en|ko] [--fields <field,...>] [--output json]

Discord notification delivery remains outbound-only. Claude inbound handling is
delegated to the official Claude Discord Channel plugin. Codex continuation is
unsupported until an official compatible capability is verified.
";

/// Non-sensitive halt fields allowed in an outbound notification.
pub(crate) struct UsageHaltNotification<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) host_scope: &'a str,
    pub(crate) selected_window: &'a str,
    /// Why the halt was recorded: it selects the recovery advice.
    pub(crate) decision: &'a str,
    pub(crate) remaining_percent: Option<f64>,
    pub(crate) measured_at: u64,
    pub(crate) evidence_digest: &'a str,
    pub(crate) run_id: Option<&'a str>,
    pub(crate) request_summary: Option<&'a str>,
    pub(crate) progress: Option<&'a str>,
    pub(crate) request_privacy: &'a str,
    pub(crate) interface_language: &'a str,
    pub(crate) message_fields: &'a [String],
}

/// Non-sensitive fields of a missing-measurement warning (OV160-013 U5). Work continues.
pub(crate) struct UsageObservationWarning<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) host_scope: &'a str,
    pub(crate) missing_windows: &'a [&'a str],
    pub(crate) cause: &'a str,
    pub(crate) capture_reason: Option<&'a str>,
    pub(crate) interface_language: &'a str,
}

/// Localized label of one usage window value.
pub(crate) fn window_display(window: &str, korean: bool) -> &'static str {
    match (window, korean) {
        ("session", true) => "세션(5시간)",
        ("session", false) => "session (5-hour)",
        ("weekly", true) => "주간",
        ("weekly", false) => "weekly",
        ("provider", true) => "제공자 한도",
        ("provider", false) => "provider quota",
        (_, true) => "알 수 없음",
        (_, false) => "unknown",
    }
}

fn host_display(host: &str) -> &str {
    match host {
        "claude" => "Claude Code",
        "codex" => "Codex",
        "antigravity" => "Antigravity",
        other => other,
    }
}

/// Localized cause of a missing measurement.
pub(crate) fn observation_cause_display(
    cause: &str,
    host: &str,
    capture_reason: Option<&str>,
    korean: bool,
) -> String {
    match (cause, korean) {
        ("no-measurement", true) if host == "claude" => {
            "상태 표시줄 기록 없음(상태 표시줄 연결 없음 또는 첫 응답 전)".to_owned()
        }
        ("no-measurement", false) if host == "claude" => {
            "no status-line capture (status line not connected, or before the first response)"
                .to_owned()
        }
        ("no-measurement", true) => "측정값 없음".to_owned(),
        ("no-measurement", false) => "no measurement".to_owned(),
        ("measurement-stale", true) => "마지막 측정이 오래됨".to_owned(),
        ("measurement-stale", false) => "the last measurement is too old".to_owned(),
        ("window-missing", true) => "호스트가 이 한도를 보고하지 않음".to_owned(),
        ("window-missing", false) => "the host did not report this window".to_owned(),
        ("capture-failed", true) => format!(
            "상태 표시줄 기록 거부({})",
            capture_reason.unwrap_or("capture-failed")
        ),
        ("capture-failed", false) => format!(
            "status-line capture rejected ({})",
            capture_reason.unwrap_or("capture-failed")
        ),
        ("fallback-required", true) => "대체 측정 동의 필요".to_owned(),
        ("fallback-required", false) => "fallback measurement needs consent".to_owned(),
        ("sensor-unavailable", true) => "측정 도구 사용 불가".to_owned(),
        ("sensor-unavailable", false) => "usage sensor unavailable".to_owned(),
        (_, true) => "측정값 확인 불가".to_owned(),
        (_, false) => "measurement could not be verified".to_owned(),
    }
}

/// The one command that makes the host's usage measurable.
pub(crate) fn observation_fix_command(host: &str) -> &'static str {
    match host {
        "claude" => "hive setup --scope user --section usage --claude-statusline preview",
        "codex" => "hive usage probe-native --host codex",
        _ => "hive setup --scope user --section usage",
    }
}

/// Dedicated missing-measurement notice. It never says that work stopped.
pub(crate) fn observation_warning_content(warning: &UsageObservationWarning<'_>) -> String {
    let korean = warning.interface_language == "ko";
    let windows = warning
        .missing_windows
        .iter()
        .map(|window| window_display(window, korean))
        .collect::<Vec<_>>()
        .join(", ");
    let cause = observation_cause_display(
        warning.cause,
        warning.host_scope,
        warning.capture_reason,
        korean,
    );
    let project = display_project_name(warning.project_name);
    let host = host_display(warning.host_scope);
    let fix = observation_fix_command(warning.host_scope);
    if korean {
        format!(
            "⚠️ **Aigent Hive 사용량 확인 경고**\n사용량을 측정하지 못해 경고 후 작업을 계속합니다.\n\n📊 **빠진 측정**\n한도: {windows}\n원인: {cause}\n\n📋 **작업 정보**\n프로젝트: {project}\n호스트: {host}\n\n🔧 **해결 방법**\n`{fix}`"
        )
    } else {
        format!(
            "⚠️ **Aigent Hive usage check warning**\nUsage could not be measured. Work continues with this warning.\n\n📊 **Missing measurement**\nwindows: {windows}\ncause: {cause}\n\n📋 **Task details**\nproject: {project}\nhost: {host}\n\n🔧 **To fix**\n`{fix}`"
        )
    }
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
        Some("test") => parse_test(&arguments[1..]).map(|parsed| test_result(&parsed)),
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
    let mut index = 0;
    while index < arguments.len() {
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| "Discord test option requires a value".to_owned())?;
        match arguments[index].as_str() {
            "--webhook-env" => webhook_env = Some(value.clone()),
            "--language" if matches!(value.as_str(), "en" | "ko") => language.clone_from(value),
            "--fields" => fields = value.split(',').map(str::to_owned).collect(),
            "--output" if value == "json" => {},
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
    if !valid_environment_name(&webhook_env) || !valid_message_fields(&fields) {
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

fn test_result(arguments: &TestArguments) -> ActionResult {
    let outcome = match env::var(&arguments.webhook_env) {
        Ok(url) => {
            let payload = payload_for(
                &UsageHaltNotification {
                    project_name: "example-project",
                    host_scope: "codex",
                    selected_window: "session",
                    decision: "halted",
                    remaining_percent: Some(20.0),
                    measured_at: 0,
                    evidence_digest: "sha256:test-notification",
                    run_id: None,
                    request_summary: None,
                    progress: None,
                    request_privacy: "summary",
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
    if !matches!(arguments.len(), 2 | 4)
        || arguments.first().map(String::as_str) != Some("--host")
        || (arguments.len() == 4
            && (arguments.get(2).map(String::as_str) != Some("--output")
                || arguments.get(3).map(String::as_str) != Some("json")))
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

/// The exact text a halt notice would send.
#[cfg(test)]
pub(crate) fn halt_content(halt: &UsageHaltNotification<'_>) -> String {
    payload_for(halt, false).content
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

/// Deliver one prepared mediation notice (OV160-006) through the same opt-in webhook path.
///
/// The caller builds the text from bounded, non-secret fields; it is cut to Discord's 2000
/// characters and mentions stay disabled. One call is one delivery attempt in the existing
/// transport sense; the caller records the outcome and never calls again for the same event.
pub(crate) fn notify_content(
    enabled: bool,
    webhook_url_env: Option<&str>,
    content: &str,
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
    notify_payload_with_url(&url, &content_payload(content), deliver_https)
}

fn content_payload(content: &str) -> DiscordPayload {
    DiscordPayload {
        content: content.chars().take(2000).collect(),
        allowed_mentions: AllowedMentions { parse: Vec::new() },
    }
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
    let request_summary = matches!(halt.request_privacy, "summary" | "raw-prompt")
        .then_some(halt.request_summary)
        .flatten();
    let mut lines = Vec::<String>::new();
    if test {
        lines.push(
            if korean {
                "🧪 **시험 알림**"
            } else {
                "🧪 **Test notification**"
            }
            .to_owned(),
        );
        lines.push(
            if korean {
                "이 형식은 자유롭게 변경을 요청할 수 있습니다."
            } else {
                "You can freely ask to change this format."
            }
            .to_owned(),
        );
        lines.push(String::new());
    }
    lines.push(
        if korean {
            "🚨 **Aigent Hive 사용량 보호 알림**"
        } else {
            "🚨 **Aigent Hive usage guard alert**"
        }
        .to_owned(),
    );
    lines.push(
        if korean {
            "작업이 중단되었습니다."
        } else {
            "Workflow stopped."
        }
        .to_owned(),
    );
    let mut previous_section = None;
    for field in halt.message_fields {
        let Some(line) = notification_field_line(halt, request_summary, field, korean) else {
            continue;
        };
        let section = match field.as_str() {
            "remaining-usage" => "usage",
            "resume" => "resume",
            _ => "task",
        };
        if previous_section != Some(section) {
            lines.push(String::new());
            lines.push(
                match (section, korean) {
                    ("usage", true) => "📊 **사용량**",
                    ("usage", false) => "📊 **Usage**",
                    ("resume", true) => "▶️ **작업을 계속하려면**",
                    ("resume", false) => "▶️ **To continue this task**",
                    (_, true) => "📋 **작업 정보**",
                    (_, false) => "📋 **Task details**",
                }
                .to_owned(),
            );
            previous_section = Some(section);
        }
        lines.push(line);
    }
    DiscordPayload {
        content: lines.join("\n"),
        allowed_mentions: AllowedMentions { parse: Vec::new() },
    }
}

fn notification_field_line(
    halt: &UsageHaltNotification<'_>,
    request_summary: Option<&str>,
    field: &str,
    korean: bool,
) -> Option<String> {
    Some(match field {
        // Claude's own screen counts up (used); Codex and Antigravity count down (remaining).
        "remaining-usage" => {
            let used = crate::usage_control::native_usage_direction(halt.host_scope) == "used";
            let label = match (used, korean) {
                (true, true) => "사용한 양",
                (true, false) => "used",
                (false, true) => "남은 사용량",
                (false, false) => "remaining usage",
            };
            let shown = halt
                .remaining_percent
                .map(|remaining| if used { 100.0 - remaining } else { remaining });
            format!("{label}: {}", display_usage_percent(shown, korean))
        }
        "project" => format!(
            "{}: {}",
            if korean { "프로젝트" } else { "project" },
            display_project_name(halt.project_name)
        ),
        // A start-of-work check has no run plan; say so instead of an empty field.
        "request" if korean && halt.run_id.is_none() => "요청: 실행 계획 없음".to_owned(),
        "request" if halt.run_id.is_none() => "request: no run plan".to_owned(),
        "request" if korean => request_summary.map_or_else(
            || "요청: 공유하지 않음(확인 가능한 작업 요약 없음)".to_owned(),
            |summary| format!("요청: {summary}"),
        ),
        "request" => request_summary.map_or_else(
            || "request: not shared (no canonical request summary is available)".to_owned(),
            |summary| format!("request: {summary}"),
        ),
        "progress" if korean && halt.run_id.is_none() => "진행 상태: 실행 계획 없음".to_owned(),
        "progress" if halt.run_id.is_none() => "progress: no run plan".to_owned(),
        "progress" if korean => halt.progress.map_or_else(
            || "진행 상태: 알 수 없음(확인 가능한 작업 진행 정보 없음)".to_owned(),
            |progress| format!("진행 상태: {progress} 완료 항목"),
        ),
        "progress" => halt.progress.map_or_else(
            || "progress: unknown (no canonical run progress is available)".to_owned(),
            |progress| format!("progress: {progress} checklist items complete"),
        ),
        "host" => format!(
            "{}: {} · {}",
            if korean { "호스트" } else { "host" },
            host_display(halt.host_scope),
            window_display(halt.selected_window, korean)
        ),
        "resume" if halt.decision == "usage-reset" => if korean {
            "사용량 초기화를 감지해 새 할당량을 지키려고 멈췄습니다. 계속하려면 이 프로젝트의 Hive 대화에서 새 요청을 보내세요."
        } else {
            "Hive stopped to protect the quota after a detected reset. To continue, send a new request in this project's Hive conversation."
        }
        .to_owned(),
        // Without a measurement there is nothing to recover above the threshold.
        "resume" if halt.remaining_percent.is_none() => {
            let fix = observation_fix_command(halt.host_scope);
            if korean {
                format!("한도 초과가 아니라 사용량을 읽지 못해 멈췄습니다. 같은 요청을 반복하기 전에 측정을 복구하세요: `{fix}`")
            } else {
                format!("Usage could not be read; this is not a reached limit. Restore the measurement before you repeat the request: `{fix}`")
            }
        }
        "resume" if crate::usage_control::native_usage_direction(halt.host_scope) == "used" => {
            used_direction_resume(halt, korean)
        }
        // Asking to continue before usage recovers only repeats the same stop, so the advice
        // names the recovery condition first.
        "resume" if korean => halt.run_id.map_or_else(
            || "사용량이 기준 위로 회복된 뒤(초기화 뒤) 이 프로젝트의 Hive 대화에서 새 요청을 보내세요. 새 요청마다 사용량을 다시 확인합니다."
                .to_owned(),
            |run_id| {
                format!(
                    "사용량이 기준 위로 회복된 뒤 이 프로젝트의 Hive 대화에서 run `{run_id}` 이어 하기를 요청하세요."
                )
            },
        ),
        "resume" => halt.run_id.map_or_else(
            || "After usage recovers above the threshold (after the reset), send a new request in this project's Hive conversation. Each new request rechecks usage."
                .to_owned(),
            |run_id| {
                format!(
                    "After usage recovers above the threshold, ask this project's Hive conversation to continue run `{run_id}`."
                )
            },
        ),
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
        _ => return None,
    })
}

/// A used number counts up, so recovery means falling below the stop point.
fn used_direction_resume(halt: &UsageHaltNotification<'_>, korean: bool) -> String {
    match (korean, halt.run_id) {
                (true, None) => "사용량이 초기화되어 사용한 양이 멈춤 기준 아래로 내려간 뒤 이 프로젝트의 Hive 대화에서 새 요청을 보내세요. 새 요청마다 사용량을 다시 확인합니다.".to_owned(),
                (true, Some(run_id)) => format!("사용량이 초기화되어 사용한 양이 멈춤 기준 아래로 내려간 뒤 이 프로젝트의 Hive 대화에서 run `{run_id}` 이어 하기를 요청하세요."),
                (false, None) => "After usage resets below the stop point, send a new request in this project's Hive conversation. Each new request rechecks usage.".to_owned(),
                (false, Some(run_id)) => format!("After usage resets below the stop point, ask this project's Hive conversation to continue run `{run_id}`."),
            }
}

fn display_usage_percent(percent: Option<f64>, korean: bool) -> String {
    percent
        .filter(|value| value.is_finite() && (0.0..=100.0).contains(value))
        .map_or_else(
            || {
                if korean {
                    "알 수 없음".to_owned()
                } else {
                    "unknown".to_owned()
                }
            },
            |value| format!("{value:.2}%"),
        )
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

    fn notification(fields: &[String]) -> UsageHaltNotification<'_> {
        UsageHaltNotification {
            project_name: "aigent-hive",
            host_scope: "codex",
            selected_window: "weekly",
            decision: "halted",
            remaining_percent: Some(18.5),
            measured_at: 1_700_000_000,
            evidence_digest: "sha256:allowed-evidence",
            run_id: None,
            request_summary: None,
            progress: None,
            request_privacy: "summary",
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
        assert!(payload.contains("progress: no run plan"));
        assert!(payload.contains("request: no run plan"));
        assert!(payload.contains("host: Codex · weekly"));
        assert!(payload.contains("remaining usage: 18.50%"));
        assert!(!payload.contains("continue the task"));
    }

    #[test]
    fn korean_halt_names_the_window_and_drops_the_looping_continue_advice() {
        let fields = default_message_fields();
        let mut notification = notification(&fields);
        notification.interface_language = "ko";
        notification.host_scope = "claude";
        notification.selected_window = "session";
        let payload = payload_for(&notification, false).content;

        assert!(payload.contains("호스트: Claude Code · 세션(5시간)"));
        assert!(payload.contains("요청: 실행 계획 없음"));
        assert!(payload.contains("진행 상태: 실행 계획 없음"));
        assert!(payload.contains("멈춤 기준 아래로 내려간 뒤"));
        assert!(!payload.contains("기준 위로"));
        assert!(!payload.contains("작업을 계속 진행해 주세요"));
        assert!(!payload.contains("unknown"));
    }

    #[test]
    fn claude_halts_show_used_and_unreadable_usage_never_promises_recovery() {
        let fields = default_message_fields();
        let mut notification = notification(&fields);
        notification.interface_language = "ko";
        notification.host_scope = "claude";
        notification.remaining_percent = Some(3.0);
        let limited = payload_for(&notification, false).content;
        assert!(limited.contains("사용한 양: 97.00%"), "{limited}");
        assert!(!limited.contains("남은 사용량"), "{limited}");

        notification.decision = "usage-unknown";
        notification.remaining_percent = None;
        let unknown = payload_for(&notification, false).content;
        assert!(unknown.contains("사용한 양: 알 수 없음"), "{unknown}");
        assert!(unknown.contains("한도 초과가 아니라"), "{unknown}");
        assert!(unknown.contains("--claude-statusline preview"), "{unknown}");
        assert!(!unknown.contains("회복된 뒤"), "{unknown}");

        notification.decision = "usage-reset";
        notification.remaining_percent = Some(100.0);
        let reset = payload_for(&notification, false).content;
        assert!(reset.contains("초기화를 감지해"), "{reset}");
        assert!(!reset.contains("회복된 뒤"), "{reset}");
    }

    #[test]
    fn missing_measurement_notice_is_a_warning_with_windows_cause_and_fix() {
        let windows = ["session", "weekly"];
        let korean = super::observation_warning_content(&super::UsageObservationWarning {
            project_name: "aigent-hive",
            host_scope: "claude",
            missing_windows: &windows,
            cause: "no-measurement",
            capture_reason: None,
            interface_language: "ko",
        });
        assert_eq!(
            korean,
            "⚠️ **Aigent Hive 사용량 확인 경고**\n사용량을 측정하지 못해 경고 후 작업을 계속합니다.\n\n📊 **빠진 측정**\n한도: 세션(5시간), 주간\n원인: 상태 표시줄 기록 없음(상태 표시줄 연결 없음 또는 첫 응답 전)\n\n📋 **작업 정보**\n프로젝트: aigent-hive\n호스트: Claude Code\n\n🔧 **해결 방법**\n`hive setup --scope user --section usage --claude-statusline preview`"
        );
        assert!(!korean.contains("중단"));
        let english = super::observation_warning_content(&super::UsageObservationWarning {
            project_name: "aigent-hive",
            host_scope: "claude",
            missing_windows: &windows[1..],
            cause: "capture-failed",
            capture_reason: Some("user-setup-missing"),
            interface_language: "en",
        });
        assert!(english.starts_with("⚠️ **Aigent Hive usage check warning**\nUsage could not be measured. Work continues with this warning."));
        assert!(english
            .contains("windows: weekly\ncause: status-line capture rejected (user-setup-missing)"));
        assert!(!english.contains("stopped"));
    }

    #[test]
    fn test_payload_matches_the_actual_payload_except_for_the_disclaimer() {
        let fields = default_message_fields();
        let actual = payload_for(&notification(&fields), false).content;
        let test = payload_for(&notification(&fields), true).content;

        assert!(actual.starts_with(
            "🚨 **Aigent Hive usage guard alert**\nWorkflow stopped.\n\n📊 **Usage**"
        ));
        assert!(actual.contains("\n\n📋 **Task details**"));
        assert!(actual.contains("\n\n▶️ **To continue this task**"));
        assert!(!actual.contains("__"));
        assert!(test.starts_with(
            "🧪 **Test notification**\nYou can freely ask to change this format.\n\n"
        ));
        assert_eq!(
            test.strip_prefix(
                "🧪 **Test notification**\nYou can freely ask to change this format.\n\n"
            ),
            Some(actual.as_str())
        );
    }

    #[test]
    fn korean_payload_uses_korean_labels_without_english_label_mixup() {
        let fields = vec!["remaining-usage".to_owned(), "project".to_owned()];
        let mut notification = notification(&fields);
        notification.interface_language = "ko";
        let payload = payload_for(&notification, true).content;

        assert!(payload
            .starts_with("🧪 **시험 알림**\n이 형식은 자유롭게 변경을 요청할 수 있습니다.\n\n"));
        assert!(payload.contains("📊 **사용량**"));
        assert!(payload.contains("📋 **작업 정보**"));
        assert!(payload.contains("남은 사용량: 18.50%"));
        assert!(payload.contains("프로젝트: aigent-hive"));
        assert!(!payload.contains("remaining usage:"));
        assert!(!payload.contains("project:"));
    }

    #[test]
    fn payload_uses_only_canonical_run_summary_and_progress_when_available() {
        let fields = vec![
            "request".to_owned(),
            "progress".to_owned(),
            "resume".to_owned(),
        ];
        let mut notification = notification(&fields);
        notification.run_id = Some("release-verify");
        notification.request_summary = Some("Verify the release package");
        notification.progress = Some("2/3");
        let payload = payload_for(&notification, false).content;

        assert!(payload.contains("request: Verify the release package"));
        assert!(payload.contains("progress: 2/3 checklist items complete"));
        assert!(payload
            .contains("ask this project's Hive conversation to continue run `release-verify`"));
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
    fn mediation_notice_uses_the_same_webhook_path_without_mentions() {
        let mut sent = Vec::new();
        let long = "가".repeat(2100);
        let outcome = super::notify_payload_with_url(
            "https://discord.com/api/webhooks/1234567890/a-valid-token",
            &super::content_payload(&long),
            |_, payload| {
                sent.push(payload.to_vec());
                Ok(())
            },
        );
        assert_eq!(outcome, NotificationOutcome::Sent);
        let payload: serde_json::Value = serde_json::from_slice(&sent[0]).unwrap();
        assert_eq!(payload["allowed_mentions"]["parse"], serde_json::json!([]));
        assert_eq!(payload["content"].as_str().unwrap().chars().count(), 2000);
        assert_eq!(
            super::notify_content(false, None, "x"),
            NotificationOutcome::Disabled
        );
        assert_eq!(
            super::notify_content(true, None, "x"),
            NotificationOutcome::MissingWebhookEnvironment
        );
    }

    #[test]
    fn codex_inbound_continuation_is_truthfully_unsupported() {
        let result = inbound_result("codex");

        assert_eq!(result.status, "unsupported");
        assert_eq!(result.code, "hive.discord-inbound-unsupported");
        assert_eq!(result.exit_code, 4);
    }
}
