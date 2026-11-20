//! Command registry, typo suggestions and readable help.
//! Suggestions use edit distance and prefixes over this table only; no model is called.
use crate::user_setup::InterfaceLanguage;
use std::fmt::Write as _;
use std::process::ExitCode;

/// Help and suggestion text stays inside a standard 80-column terminal.
pub(crate) const COLUMNS: usize = 80;
const MAX_SUGGESTIONS: usize = 3;
/// Options every command accepts before its own parser runs.
const GLOBAL_OPTIONS: &[&str] = &["--help", "--output", "--quiet", "--version"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Group {
    Start,
    Setup,
    Knowledge,
    Update,
    Diagnose,
    Advanced,
}

const GROUPS: [(Group, &str, &str); 6] = [
    (Group::Start, "시작하기", "Getting started"),
    (Group::Setup, "설정", "Configuration"),
    (Group::Knowledge, "지식", "Knowledge"),
    (Group::Update, "갱신", "Updates"),
    (Group::Diagnose, "진단", "Diagnostics"),
    (Group::Advanced, "고급", "Advanced"),
];

pub(crate) struct Command {
    pub(crate) name: &'static str,
    group: Group,
    ko: &'static str,
    en: &'static str,
    /// Closed first-word actions. Empty means the command takes no subcommand check.
    pub(crate) subcommands: &'static [&'static str],
    /// Every option the command's parsers accept; a drift test reads the parser sources.
    pub(crate) options: &'static [&'static str],
    /// The first example is the default. A later one is chosen when it names the subcommand.
    examples: &'static [&'static str],
}

const fn command(
    name: &'static str,
    group: Group,
    ko: &'static str,
    en: &'static str,
    subcommands: &'static [&'static str],
    options: &'static [&'static str],
    examples: &'static [&'static str],
) -> Command {
    Command {
        name,
        group,
        ko,
        en,
        subcommands,
        options,
        examples,
    }
}

const SETUP_OPTIONS: &[&str] = &[
    "--answer",
    "--answers",
    "--apply",
    "--capabilities",
    "--describe",
    "--dry-run",
    "--id",
    "--index-rebuild-scope",
    "--input-channel",
    "--interaction",
    "--interactive",
    "--monitor-confirm",
    "--non-interactive",
    "--progress",
    "--question",
    "--quick-answers",
    "--reconfigure-role",
    "--resume",
    "--review",
    "--review-answers",
    "--scope",
    "--section",
    "--show",
    "--step",
    "--summary",
    "--target",
    "--user-root",
    "--validate",
];
const INSTALL_OPTIONS: &[&str] = &[
    "--apply",
    "--dry-run",
    "--host",
    "--hosts",
    "--recover",
    "--scope",
    "--user-root",
    "--validate",
];
const UNINSTALL_OPTIONS: &[&str] = &["--confirm", "--dry-run", "--resume", "--user-root"];
const UPDATE_OPTIONS: &[&str] = &[
    "--apply",
    "--bundle",
    "--channel",
    "--check",
    "--confirm",
    "--direct-handoff",
    "--dry-run",
    "--exact-major-target",
    "--host",
    "--hosts",
    "--interactive",
    "--major-confirmation",
    "--non-interactive",
    "--recover",
    "--scope",
    "--target",
    "--user-root",
    "--validate",
];
const KNOWLEDGE_OPTIONS: &[&str] = &[
    "--answer",
    "--apply",
    "--authorization-id",
    "--authorization-token",
    "--batch",
    "--bundle",
    "--byte-budget",
    "--candidates",
    "--capabilities",
    "--capability",
    "--category",
    "--claim-key",
    "--code-only",
    "--collection",
    "--collections",
    "--confidential-collection",
    "--confirm-current-action",
    "--confirm-global-promotion",
    "--confirm-subject-assignment",
    "--confirm-subject-restore",
    "--consent-digest",
    "--dry-run",
    "--engine",
    "--exclude-conflicts",
    "--expansions",
    "--expected-sha256",
    "--expected-source-digest",
    "--expires-at",
    "--fingerprint",
    "--force",
    "--format",
    "--host",
    "--id",
    "--include-untracked",
    "--input",
    "--inventory",
    "--kind",
    "--language",
    "--limit",
    "--max-seconds",
    "--mode",
    "--no-cluster",
    "--node-id",
    "--nonce",
    "--offset",
    "--operation",
    "--page-id",
    "--preview",
    "--preview-digest",
    "--prior-inventory",
    "--python",
    "--query",
    "--quick",
    "--reason",
    "--rebuild-mode",
    "--receipt",
    "--receipt-digest",
    "--replace-backup",
    "--replacement",
    "--request",
    "--request-digest",
    "--review",
    "--review-digest",
    "--review-id",
    "--scope",
    "--snapshot",
    "--source",
    "--source-locator",
    "--subject",
    "--tag",
    "--target",
    "--text",
    "--timestamp",
    "--top-k",
    "--usage",
    "--user-root",
    "--user-statement",
    "--visibility",
    "--wiki",
    "--workers",
    "--write-receipt",
];
/// Source Wiki shares the knowledge graph and vector parsers.
const SOURCE_WIKI_OPTIONS: &[&str] = &[
    "--authorization-id",
    "--authorization-token",
    "--capabilities",
    "--collection",
    "--collections",
    "--confirm-current-action",
    "--consent-digest",
    "--engine",
    "--expires-at",
    "--format",
    "--host",
    "--input",
    "--language",
    "--limit",
    "--max-seconds",
    "--node-id",
    "--nonce",
    "--operation",
    "--python",
    "--query",
    "--rebuild-mode",
    "--receipt",
    "--request-digest",
    "--scope",
    "--tag",
    "--target",
    "--text",
    "--top-k",
    "--usage",
    "--user-root",
    "--visibility",
    "--workers",
];
const USAGE_OPTIONS: &[&str] = &[
    "--account-digest",
    "--action",
    "--apply",
    "--confirm",
    "--confirm-install",
    "--confirm-reset-guard-disable",
    "--confirm-session-disable",
    "--confirm-session-interruption",
    "--confirm-zero-quota-guard-disable",
    "--dry-run",
    "--host",
    "--process-id",
    "--remaining-percent",
    "--request",
    "--run",
    "--session-id",
    "--stdin-json",
    "--target",
    "--target-from-stdin",
    "--threshold",
    "--user-root",
];
const PROJECT_OPTIONS: &[&str] = &[
    "--apply",
    "--approve-skill-merge",
    "--dry-run",
    "--recover",
    "--scan",
    "--skill-merge-inputs",
    "--skill-merges",
    "--target",
    "--user-root",
    "--validate",
];
const AGENT_OPTIONS: &[&str] = &[
    "--accept-decision-digest",
    "--accept-definition-digest",
    "--capabilities",
    "--catalog",
    "--catalog-attestation",
    "--host",
    "--previous-request",
    "--prior-decision",
    "--profile",
    "--project-root",
    "--purpose",
    "--receipt",
    "--request",
    "--role",
    "--root",
    "--scope",
    "--trust-root",
    "--user-root",
];
const ORCHESTRATION_OPTIONS: &[&str] = &[
    "--action",
    "--apply",
    "--authority",
    "--authority-id",
    "--capabilities",
    "--control-epoch",
    "--dry-run",
    "--emergency",
    "--event",
    "--expected-head",
    "--from-run",
    "--grant",
    "--now",
    "--receipt",
    "--recover",
    "--request-digest",
    "--run",
    "--subject-authority",
    "--target",
    "--task",
    "--trust-root",
    "--user-root",
];
const MSG_OPTIONS: &[&str] = &[
    "--advisor",
    "--allow-same-provider",
    "--binding",
    "--builder",
    "--confirm",
    "--effort",
    "--event",
    "--file",
    "--flow",
    "--from",
    "--host",
    "--id",
    "--key",
    "--kind",
    "--lease",
    "--message",
    "--model",
    "--note-file",
    "--now",
    "--option",
    "--provider",
    "--question-file",
    "--recommend",
    "--result",
    "--return",
    "--reviewer",
    "--role",
    "--state",
    "--target",
    "--task",
    "--to",
    "--ttl",
    "--user-root",
];
const SUMMON_OPTIONS: &[&str] = &[
    "--action",
    "--authority",
    "--authority-id",
    "--capabilities",
    "--control-epoch",
    "--event",
    "--expected-head",
    "--now",
    "--request-digest",
    "--run",
    "--target",
    "--task",
    "--trust-root",
    "--user-root",
];
const RUN_OPTIONS: &[&str] = &[
    "--account-digest",
    "--candidate",
    "--capabilities",
    "--claim-nudge",
    "--class",
    "--confirm",
    "--dispatch-intent",
    "--evaluation",
    "--process-id",
    "--request",
    "--role",
    "--rule",
    "--run",
    "--session-id",
    "--target",
    "--threshold",
    "--user-root",
];
const POLICY_OPTIONS: &[&str] = &[
    "--confirm",
    "--event",
    "--expected-context",
    "--expected-policy",
    "--host",
    "--input",
    "--review-run",
    "--stdin-json",
    "--target",
];
const KOREAN_OPTIONS: &[&str] = &[
    "--after",
    "--before",
    "--candidate",
    "--confirm-pack",
    "--consent-digest",
    "--input",
    "--output-file",
    "--profile",
    "--target",
];

#[rustfmt::skip]
pub(crate) const COMMANDS: &[Command] = &[
    command("install", Group::Start, "사용자 환경에 Hive 설치·확인", "Install or check Hive for your user",
        &[], INSTALL_OPTIONS,
        &["hive install --scope user --hosts codex,claude --dry-run",
          "hive install --scope user --hosts codex,claude --apply",
          "hive install --scope user --hosts codex,claude --validate"]),
    command("setup", Group::Start, "처음 설정 또는 프로젝트 설정", "First-time or project setup",
        &["feature"], SETUP_OPTIONS,
        &["hive setup", "hive setup --target .", "hive setup --target . --show",
          "hive setup feature status --id vector-search"]),
    command("doctor", Group::Start, "설치 상태 확인", "Inspect the installation",
        &[], &[], &["hive doctor"]),
    command("check-target", Group::Start, "프로젝트 폴더 사용 가능 여부 확인", "Check a project folder",
        &[], &[], &["hive check-target ."]),
    command("project", Group::Setup, "프로젝트 Hive 지침 갱신", "Upgrade a project's Hive harness",
        &["upgrade"], PROJECT_OPTIONS,
        &["hive project upgrade --target . --scan",
          "hive project upgrade --target . --dry-run",
          "hive project upgrade --target . --apply"]),
    command("agent", Group::Setup, "사용자 정의 에이전트 관리", "Manage custom agents",
        &["preview", "recommend", "create", "apply", "validate", "preflight", "attest", "route",
          "disable", "remove"], AGENT_OPTIONS, &["hive agent --help"]),
    command("role", Group::Setup, "역할 확인과 인계 기록", "Validate roles and record handoffs",
        &["validate", "handoff"], &["--request", "--role", "--target"],
        &["hive role validate --target . --role <role-id>"]),
    command("roster", Group::Setup, "신 이름 목록과 역할 배정 보기", "Show named roles and assignments",
        &["check"], &["--capabilities", "--role", "--target", "--user-root"],
        &["hive roster --target . --user-root <user-root>"]),
    command("summon", Group::Setup, "신 이름으로 기능 호출", "Call a function by its named role",
        &[], SUMMON_OPTIONS, &["hive summon --help"]),
    command("msg", Group::Setup, "역할 사이 메시지 보내기·받기", "Send and read role messages",
        &["send", "inbox", "claim", "ack", "status", "flow", "decide", "hooks", "hook"], MSG_OPTIONS,
        &["hive msg inbox --target . --user-root ~",
          "hive msg send --to artemis --from athena --file a.md --target . --user-root ~",
          "hive msg status --id <message-id> --target . --user-root ~",
          "hive msg flow start --builder hephaistos --reviewer athena --task t1 --file a.md --target . --user-root ~",
          "hive msg hooks preview --host claude --role athena --target . --user-root ~",
          "hive msg decide list --state open --target . --user-root ~"]),
    command("knowledge", Group::Knowledge, "지식 저장·검색·관리", "Save, find and manage knowledge",
        &["ingest", "add", "query", "graph", "list", "read", "promote", "lint", "delete",
          "suppress", "remember", "retrieve", "subjects", "authorize-confidential",
          "authorize-collection", "collection", "refresh", "scan", "export", "import", "transfer",
          "vector"], KNOWLEDGE_OPTIONS,
        &["hive knowledge lint --target .",
          "hive knowledge query --target . --text build",
          "hive knowledge retrieve --user-root <user-root> --target . --query build",
          "hive knowledge remember --user-root <user-root> --user-statement \"Use pnpm\" --claim-key build-tool",
          "hive knowledge list --target ."]),
    command("source-wiki", Group::Knowledge, "Hive 소스 위키 검사·검색", "Check or search the Hive source Wiki",
        &["lint", "index", "query", "graph", "vector"], SOURCE_WIKI_OPTIONS,
        &["hive source-wiki lint --target .",
          "hive source-wiki query --target . --language ko --text help"]),
    command("index", Group::Knowledge, "지식 검색 색인 다시 만들기", "Rebuild the knowledge index",
        &["rebuild"], &["--target", "--user-root"], &["hive index rebuild --target ."]),
    command("update", Group::Update, "Hive 갱신 또는 새 버전 확인", "Update Hive or check for a new version",
        &[], UPDATE_OPTIONS,
        &["hive update", "hive update --channel test --confirm",
          "hive update --check --user-root <user-root>"]),
    command("uninstall", Group::Update, "사용자 환경에서 Hive 제거", "Remove Hive from your user",
        &[], UNINSTALL_OPTIONS, &["hive uninstall --dry-run"]),
    command("release", Group::Update, "내려받은 릴리스 묶음 검증", "Verify a downloaded release bundle",
        &["verify"], &["--bundle"], &["hive release verify --bundle <release-dir>"]),
    command("usage", Group::Diagnose, "사용량 확인과 사용량 보호", "Check usage and the usage guard",
        &["check", "probe-native", "monitor", "show", "enforce", "status", "threshold", "session",
          "capture", "summarize-tokens", "fallback-install"], USAGE_OPTIONS,
        &["hive usage threshold --user-root <user-root> --remaining-percent 20",
          "hive usage monitor status --user-root <user-root>",
          "hive usage status --target . --session-id <id> --process-id <pid>",
          "hive usage show --host codex --user-root <user-root>"]),
    command("storage", Group::Diagnose, "Hive 저장 공간 측정", "Measure Hive storage",
        &[], &["--confirm", "--dry-run", "--user-root"], &["hive storage"]),
    command("prune", Group::Diagnose, "정리할 파일 미리 보기·삭제", "Preview or remove old files",
        &[], &["--confirm", "--dry-run", "--user-root"], &["hive prune --dry-run"]),
    command("report", Group::Diagnose, "문제 보고 자료 만들기", "Prepare a problem report",
        &["preview", "collect", "export"],
        &["--destination", "--issue-code", "--report", "--summary", "--target"],
        &["hive report preview --target ."]),
    command("korean", Group::Diagnose, "한국어 출력 검사", "Check Korean output",
        &["inspect", "verify", "sanitize", "pack"], KOREAN_OPTIONS, &["hive korean --help"]),
    command("session", Group::Advanced, "자동 편집 세션 예약·확인", "Reserve or check edit sessions",
        &["begin", "check", "update", "close", "recover"],
        &["--host", "--path", "--process-id", "--session-id", "--target"],
        &["hive session recover --target ."]),
    command("run", Group::Advanced, "작업 체크포인트·재개·종료 확인", "Checkpoint, resume or close work",
        &["checkpoint", "resume", "closure", "continuation", "policy-review"], RUN_OPTIONS,
        &["hive run closure --target . --run <run-id>"]),
    command("loop", Group::Advanced, "반복 작업 그래프 관리", "Manage loop work graphs",
        &["initialize", "validate", "checkpoint", "steer", "prepare", "recover"],
        &["--graph", "--request", "--run", "--target"], &["hive loop --help"]),
    command("orchestration", Group::Advanced, "여러 작업 배정·조율", "Assign and coordinate work",
        &["status", "plan", "dispatch", "receipt", "cancel", "recover", "authority", "migrate"],
        ORCHESTRATION_OPTIONS, &["hive orchestration status --target ."]),
    command("judge", Group::Advanced, "독립 검증 패키지·판정", "Package and judge independent checks",
        &["package", "receipt", "quorum"],
        &["--assignment", "--package", "--request", "--target", "--trust-root"],
        &["hive judge --help"]),
    command("advanced", Group::Advanced, "요청 경로 결정·프롬프트 검증 (에이전트용)",
        "Route requests or check prompts (for agents)",
        &["route", "prompt"],
        &["--confirm-refined-prompt", "--digest", "--request", "--result", "--target-host"],
        &["hive advanced route --request <request.json>",
          "hive advanced prompt validate --request <input.json> --result <result.json>"]),
    command("hook", Group::Advanced, "호스트 hook 처리 (호스트 전용)", "Handle host hooks (hosts only)",
        &[], &["--capabilities", "--capability", "--event", "--input"],
        &["hive hook --help"]),
    command("policy", Group::Advanced, "정책 평가·hook 설정", "Evaluate policy or configure hooks",
        &["evaluate", "hook", "hooks"], POLICY_OPTIONS, &["hive policy --help"]),
    command("discord", Group::Advanced, "Discord 알림 시험·수신", "Test or receive Discord notices",
        &["inbound", "test"], &["--fields", "--host", "--language", "--webhook-env"],
        &["hive discord --help"]),
];

pub(crate) fn find(name: &str) -> Option<&'static Command> {
    COMMANDS.iter().find(|command| command.name == name)
}

/// Top-level spellings retired in 0.16.0. Each now runs only as `hive advanced <name>`.
/// No shipped hook, Skill, template or installer calls them, so the old spelling only explains
/// the move and never runs the command.
pub(crate) const MOVED_TO_ADVANCED: &[&str] = &["route", "prompt"];

/// Explains a retired top-level command and exits 2 without running it.
pub(crate) fn moved_command(arguments: &[String]) -> Option<ExitCode> {
    let name = arguments.first()?.as_str();
    if !MOVED_TO_ADVANCED.contains(&name) {
        return None;
    }
    let replacement = format!("hive advanced {name}");
    let example = moved_example(name);
    if crate::human_output::machine() {
        let result = crate::ActionResult {
            schema_version: 1,
            action: "UnknownAction",
            status: "error",
            exit_code: 2,
            code: "hive.command-moved",
            message: format!("hive {name} moved to {replacement}"),
            changed_paths: Vec::new(),
            evidence: Vec::new(),
            next_action: Some(example.to_owned()),
            data: Some(serde_json::json!({
                "retired_command": format!("hive {name}"),
                "replacement": replacement,
                "help_command": "hive advanced --help",
            })),
        };
        return Some(crate::emit_action_result(&result));
    }
    print!(
        "{}",
        moved_text(name, example, crate::human_output::language())
    );
    Some(ExitCode::from(2))
}

/// The `hive advanced` example for a moved command.
fn moved_example(name: &str) -> &'static str {
    let prefix = format!("hive advanced {name} ");
    find("advanced")
        .and_then(|entry| {
            entry
                .examples
                .iter()
                .find(|example| example.starts_with(&prefix))
        })
        .copied()
        .unwrap_or("hive advanced --help")
}

fn moved_text(name: &str, example: &str, language: InterfaceLanguage) -> String {
    if language == InterfaceLanguage::Ko {
        format!(
            "처리 실패: hive {name} 명령은 hive advanced {name}로 옮겨졌습니다.\n\
             예시: {example}\n\
             다음 단계: hive advanced --help 로 옵션과 예시를 확인하세요.\n"
        )
    } else {
        format!(
            "Failed. hive {name} moved to hive advanced {name}.\n\
             Example: {example}\n\
             Next: see hive advanced --help for options and examples.\n"
        )
    }
}

/// Every option some Hive parser accepts. Only a token outside this set is rejected early,
/// so a registry gap can cost a suggestion but never blocks a valid command.
fn known_anywhere(option: &str) -> bool {
    GLOBAL_OPTIONS.contains(&option)
        || COMMANDS
            .iter()
            .any(|command| command.options.contains(&option))
}

fn looks_like_option(argument: &str) -> bool {
    argument.strip_prefix("--").is_some_and(|name| {
        name.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    })
}

/// Optimal string alignment distance: insert, delete, substitute or swap neighbours.
fn distance(left: &str, right: &str) -> usize {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    let mut rows = vec![vec![0usize; right.len() + 1]; left.len() + 1];
    for (index, row) in rows.iter_mut().enumerate() {
        row[0] = index;
    }
    for (index, cell) in rows[0].iter_mut().enumerate() {
        *cell = index;
    }
    for i in 1..=left.len() {
        for j in 1..=right.len() {
            let cost = usize::from(left[i - 1] != right[j - 1]);
            let mut best = (rows[i - 1][j] + 1)
                .min(rows[i][j - 1] + 1)
                .min(rows[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && left[i - 1] == right[j - 2] && left[i - 2] == right[j - 1] {
                best = best.min(rows[i - 2][j - 2] + 1);
            }
            rows[i][j] = best;
        }
    }
    rows[left.len()][right.len()]
}

/// Allowed edits grow with the typed length so short words do not match everything.
fn limit(length: usize) -> usize {
    match length {
        0..=3 => 1,
        4..=6 => 2,
        _ => 3,
    }
}

/// At most three real candidates, closest first. Leading dashes are ignored when comparing.
pub(crate) fn suggest<'a>(input: &str, candidates: &[&'a str]) -> Vec<&'a str> {
    let typed = input.trim_start_matches('-').to_ascii_lowercase();
    if typed.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(usize, &str)> = candidates
        .iter()
        .filter(|candidate| **candidate != input)
        .filter_map(|candidate| {
            let name = candidate.trim_start_matches('-');
            let prefix = typed.len() >= 2 && name.starts_with(&typed);
            let edits = distance(&typed, name);
            if prefix {
                Some((0, *candidate))
            } else {
                (edits <= limit(typed.chars().count())).then_some((edits, *candidate))
            }
        })
        .collect();
    scored.sort_unstable();
    scored.dedup_by(|a, b| a.1 == b.1);
    scored
        .into_iter()
        .take(MAX_SUGGESTIONS)
        .map(|(_, candidate)| candidate)
        .collect()
}

fn top_level_candidates() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = COMMANDS.iter().map(|command| command.name).collect();
    names.extend(hive_core::gods::GODS.iter().map(|god| god.name));
    names.extend(["--help", "--version"]);
    names
}

/// The example for a command, preferring one that uses the named subcommand.
pub(crate) fn example_for(command: &str, subcommand: Option<&str>) -> Option<&'static str> {
    let entry = find(command)?;
    subcommand
        .and_then(|sub| {
            let prefix = format!("hive {command} {sub} ");
            entry
                .examples
                .iter()
                .find(|example| example.starts_with(&prefix))
        })
        .or_else(|| entry.examples.first())
        .copied()
}

pub(crate) fn help_command(command: &str) -> String {
    if command == "project" {
        "hive project upgrade --help".to_owned()
    } else {
        format!("hive {command} --help")
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Problem {
    Command,
    Subcommand,
    Option,
}

struct Rejection<'a> {
    problem: Problem,
    command: Option<&'static Command>,
    input: &'a str,
    suggestions: Vec<&'static str>,
    example: Option<String>,
    help: String,
}

/// Rejects an unknown command, subcommand or option before a parser runs, with suggestions.
/// An unknown subcommand is left to its parser unless a close candidate exists.
pub(crate) fn precheck(arguments: &[String]) -> Option<ExitCode> {
    let first = arguments.first()?;
    if first.starts_with('-') || first.starts_with("__") {
        return None;
    }
    let Some(command) = find(first) else {
        let suggestions = suggest(first, &top_level_candidates());
        let (example, help) = match suggestions.first() {
            Some(name) if name.starts_with("--") => {
                (Some(format!("hive {name}")), "hive --help".to_owned())
            }
            Some(name) => match find(name) {
                Some(entry) => (
                    entry.examples.first().map(|e| (*e).to_owned()),
                    help_command(name),
                ),
                // A named role (god) has its own help.
                None => (None, format!("hive {name} --help")),
            },
            None => (None, "hive --help".to_owned()),
        };
        return Some(reject(&Rejection {
            problem: Problem::Command,
            command: None,
            input: first,
            suggestions,
            example,
            help,
        }));
    };
    // Host wire protocols keep their own exact result shapes.
    if crate::human_output::machine_protocol(arguments) {
        return None;
    }
    let rest = &arguments[1..];
    let subcommand = rest
        .first()
        .map(String::as_str)
        .filter(|s| !s.starts_with('-'));
    if let Some(sub) = subcommand
        .filter(|sub| !command.subcommands.is_empty() && !command.subcommands.contains(sub))
    {
        let suggestions = suggest(sub, command.subcommands);
        if !suggestions.is_empty() {
            return Some(reject(&Rejection {
                problem: Problem::Subcommand,
                command: Some(command),
                input: sub,
                example: example_for(command.name, suggestions.first().copied()).map(str::to_owned),
                suggestions,
                help: help_command(command.name),
            }));
        }
    }
    // An option-looking word right after a known option may be that option's value (for example
    // `--text --verbose`), so only the command's own parser may reject it.
    let unknown = rest.iter().enumerate().find_map(|(index, argument)| {
        let after_known_option = index
            .checked_sub(1)
            .and_then(|previous| rest.get(previous))
            .is_some_and(|previous| looks_like_option(previous) && known_anywhere(previous));
        (looks_like_option(argument) && !known_anywhere(argument) && !after_known_option)
            .then_some(argument)
    })?;
    let mut candidates: Vec<&'static str> = command.options.to_vec();
    candidates.extend(
        GLOBAL_OPTIONS
            .iter()
            .filter(|o| **o != "--quiet" && **o != "--version"),
    );
    let known_sub = subcommand.filter(|sub| command.subcommands.contains(sub));
    Some(reject(&Rejection {
        problem: Problem::Option,
        command: Some(command),
        input: unknown,
        suggestions: suggest(unknown, &candidates),
        example: example_for(command.name, known_sub).map(str::to_owned),
        help: help_command(command.name),
    }))
}

fn reject(rejection: &Rejection) -> ExitCode {
    let name = rejection.command.map_or("", |command| command.name);
    let message = match rejection.problem {
        Problem::Command => format!("unknown top-level action: {}", rejection.input),
        Problem::Subcommand => format!("unknown {name} action: {}", rejection.input),
        Problem::Option => format!("unknown option: {}", rejection.input),
    };
    if crate::human_output::machine() {
        let result = crate::ActionResult {
            schema_version: 1,
            action: "UnknownAction",
            status: "error",
            exit_code: 2,
            code: if rejection.problem == Problem::Command {
                "hive.unknown-action"
            } else {
                "hive.invalid-input"
            },
            message,
            changed_paths: Vec::new(),
            evidence: Vec::new(),
            next_action: rejection.example.clone(),
            data: Some(serde_json::json!({
                "suggestions": rejection.suggestions,
                "help_command": rejection.help,
            })),
        };
        return crate::emit_action_result(&result);
    }
    print!(
        "{}",
        rejection_text(rejection, crate::human_output::language())
    );
    ExitCode::from(2)
}

fn rejection_text(rejection: &Rejection, language: InterfaceLanguage) -> String {
    let ko = language == InterfaceLanguage::Ko;
    let name = rejection.command.map_or("", |command| command.name);
    let input = rejection.input;
    let mut text = match (rejection.problem, ko) {
        (Problem::Command, true) => format!("처리 실패: 알 수 없는 명령입니다: {input}\n"),
        (Problem::Command, false) => format!("Failed. Unknown command: {input}\n"),
        (Problem::Subcommand, true) => {
            format!("처리 실패: hive {name}에 없는 하위 명령입니다: {input}\n")
        }
        (Problem::Subcommand, false) => format!("Failed. hive {name} has no action {input}\n"),
        (Problem::Option, true) => format!("처리 실패: hive {name}에 없는 옵션입니다: {input}\n"),
        (Problem::Option, false) => format!("Failed. hive {name} has no option {input}\n"),
    };
    if !rejection.suggestions.is_empty() {
        let list = rejection.suggestions.join(", ");
        if ko {
            writeln!(text, "혹시 이것인가요? {list}").expect("formatting a String");
        } else {
            writeln!(text, "Did you mean? {list}").expect("formatting a String");
        }
    }
    if let Some(example) = &rejection.example {
        if ko {
            writeln!(text, "예시: {example}").expect("formatting a String");
        } else {
            writeln!(text, "Example: {example}").expect("formatting a String");
        }
    }
    let help = &rejection.help;
    if ko {
        writeln!(text, "다음 단계: {help} 로 옵션과 예시를 확인하세요.")
    } else {
        writeln!(text, "Next: see {help} for options and examples.")
    }
    .expect("formatting a String");
    text
}

/// Display width with Hangul and other wide characters counted as two columns.
pub(crate) fn width(text: &str) -> usize {
    text.chars()
        .map(|c| {
            if matches!(u32::from(c),
                0x1100..=0x115F | 0x2E80..=0xA4CF | 0xAC00..=0xD7A3 | 0xF900..=0xFAFF
                    | 0xFE30..=0xFE4F | 0xFF00..=0xFF60 | 0xFFE0..=0xFFE6)
            {
                2
            } else {
                1
            }
        })
        .sum()
}

/// Pads with spaces to a display width.
fn pad(text: &str, columns: usize) -> String {
    format!("{text}{}", " ".repeat(columns.saturating_sub(width(text))))
}

/// Wraps one line at spaces. A wrapped `hive ...` syntax line continues four columns deeper.
pub(crate) fn wrap_into(line: &str, output: &mut String) {
    if width(line) <= COLUMNS {
        output.push_str(line);
        output.push('\n');
        return;
    }
    let body = line.trim_start();
    let indent = &line[..line.len() - body.len()];
    let continuation = if body.starts_with("hive ") {
        format!("{indent}    ")
    } else {
        indent.to_owned()
    };
    let mut current = indent.to_owned();
    let mut empty = true;
    for word in body.split(' ').filter(|word| !word.is_empty()) {
        if !empty && width(&current) + 1 + width(word) > COLUMNS {
            output.push_str(&current);
            output.push('\n');
            current.clone_from(&continuation);
            empty = true;
        }
        if !empty {
            current.push(' ');
        }
        current.push_str(word);
        empty = false;
    }
    output.push_str(&current);
    output.push('\n');
}

pub(crate) fn wrap(text: &str) -> String {
    let mut output = String::new();
    for line in text.lines() {
        wrap_into(line, &mut output);
    }
    output
}

pub(crate) fn top_help(language: InterfaceLanguage) -> String {
    let ko = language == InterfaceLanguage::Ko;
    let mut text = if ko {
        "Aigent Hive: AI 코딩 도구에 지침·지식·안전 장치를 더하는 로컬 도우미

사용법: hive <명령> [옵션]
명령별 옵션 표와 예시: hive <명령> --help
결과는 저장한 언어로 표시. 기계용 출력: --output json
"
    } else {
        "Aigent Hive: a local helper that adds rules, knowledge and safety to AI tools

Usage: hive <command> [options]
Options and examples for one command: hive <command> --help
Results use your saved language. Machine output: --output json
"
    }
    .to_owned();
    for (group, ko_title, en_title) in GROUPS {
        text.push('\n');
        text.push_str(if ko { ko_title } else { en_title });
        text.push('\n');
        for command in COMMANDS.iter().filter(|command| command.group == group) {
            let summary = if ko { command.ko } else { command.en };
            writeln!(text, "  {} {summary}", pad(command.name, 15)).expect("formatting a String");
        }
        if group == Group::Setup {
            let (name, summary) = if ko {
                (
                    "<신 이름>",
                    "신 이름으로 Skill 기능 실행 (hive roster --help)",
                )
            } else {
                (
                    "<god>",
                    "Run a Skill by its named role (hive roster --help)",
                )
            };
            writeln!(text, "  {} {summary}", pad(name, 15)).expect("formatting a String");
        }
    }
    text.push_str(if ko {
        "\n버전: hive --version\n"
    } else {
        "\nVersion: hive --version\n"
    });
    text
}

type Row = (&'static str, &'static str, &'static str);

struct Detailed {
    title: &'static str,
    summary: (&'static str, &'static str),
    about: (&'static str, &'static str),
    required: &'static [Row],
    optional: &'static [Row],
    examples: &'static [&'static str],
}

#[rustfmt::skip]
const INSTALL_HELP: Detailed = Detailed {
    title: "hive install",
    summary: ("사용자 환경에 Hive 설치·확인", "install or check Hive for your user"),
    about: ("사용자 폴더에 Hive를 설치하고 고른 AI 도구(호스트)에 연결.\n먼저 --dry-run으로 바뀔 내용을 본 뒤 --apply로 적용.",
            "Installs Hive in your user folder and connects the chosen AI tools (hosts).\nPreview with --dry-run first, then apply with --apply."),
    required: &[
        ("--scope user", "설치 범위 (항상 user)", "Install scope (always user)"),
        ("--hosts <목록>", "쉼표로 구분한 호스트 목록", "Comma-separated hosts"),
        ("", "codex, claude, antigravity 중 선택", "Choose from codex, claude, antigravity"),
        ("", "또는 --host <호스트>를 여러 번", "Or repeat --host <host>"),
        ("--dry-run|--apply|--validate", "미리 보기 | 적용 | 설치 확인", "Preview | apply | check the install"),
    ],
    optional: &[
        ("--recover", "중단된 설치 복구", "Recover an interrupted install"),
        ("--user-root <폴더>", "사용자 폴더 (기본: 홈 폴더)", "User folder (default: home)"),
        ("--output json", "기계용 JSON 출력", "Machine-readable JSON output"),
    ],
    examples: &["hive install --scope user --hosts codex,claude --dry-run",
                "hive install --scope user --hosts codex,claude --apply"],
};

#[rustfmt::skip]
const SETUP_HELP: Detailed = Detailed {
    title: "hive setup",
    summary: ("처음 설정 또는 프로젝트 설정", "first-time or project setup"),
    about: ("옵션 없이 실행하면 사용자 전역 설정을 질문으로 진행.\n--target을 주면 그 프로젝트 폴더를 설정.",
            "Without options, asks the questions for your global user setup.\nWith --target, sets up that project folder."),
    required: &[
        ("--target <폴더>", "설정할 프로젝트 폴더 (프로젝트 설정)", "Project folder (project setup)"),
    ],
    optional: &[
        ("--user-root <폴더>", "사용자 폴더 (기본: 홈 폴더)", "User folder (default: home)"),
        ("--interactive", "터미널에서 질문으로 진행", "Ask questions in the terminal"),
        ("--non-interactive", "질문 없이 진행", "Never wait for input"),
        ("--resume", "중단한 설정 이어서 진행", "Continue an interrupted setup"),
        ("--show [--summary]", "현재 설정 보기 (짧은 요약)", "Show the setup (short summary)"),
        ("--describe --section <id>", "설정 항목 설명 보기", "Describe one setup section"),
        ("--answers <yml>", "답변 파일로 설정 (모드 필요)", "Use an answers file (needs a mode)"),
        ("--dry-run|--apply|--validate", "미리 보기 | 적용 | 확인", "Preview | apply | validate"),
        ("--capabilities <json>", "호스트 기능 정보 파일", "Host capability file"),
        ("--output json", "기계용 JSON 출력", "Machine-readable JSON output"),
    ],
    examples: &["hive setup", "hive setup --target .", "hive setup --target . --show"],
};

#[rustfmt::skip]
const UPDATE_HELP: Detailed = Detailed {
    title: "hive update",
    summary: ("Hive 갱신 또는 새 버전 확인", "update Hive or check for a new version"),
    about: ("설치한 Hive와 사용자 환경 연결을 새 버전으로 갱신.\n기본 채널은 stable. 시험판은 --channel test를 직접 지정.",
            "Updates the installed Hive package and its user connections.\nThe default channel is stable. The test channel needs --channel test."),
    required: &[],
    optional: &[
        ("--channel stable|test", "갱신 채널 (기본 stable)", "Update channel (default stable)"),
        ("--confirm", "소프트웨어 갱신만 승인", "Accept only the software update"),
        ("", "설정 질문에는 답하지 않음", "Does not answer setup questions"),
        ("--interactive", "터미널에서 질문 (JSON과 함께 불가)", "Ask in the terminal (not with JSON)"),
        ("--non-interactive", "입력을 기다리지 않음", "Never wait for input"),
        ("--check", "설치 없이 새 버전만 확인", "Only check for a newer version"),
        ("--user-root <폴더>", "사용자 폴더 (--check에는 필수)", "User folder (required with --check)"),
        ("--output json", "기계용 JSON 출력", "Machine-readable JSON output"),
    ],
    examples: &["hive update", "hive update --channel test --confirm",
                "hive update --check --user-root <user-root>"],
};

#[rustfmt::skip]
const KNOWLEDGE_HELP: Detailed = Detailed {
    title: "hive knowledge",
    summary: ("지식 저장·검색·관리", "save, find and manage knowledge"),
    about: ("프로젝트와 사용자 지식을 Markdown 원본으로 저장하고 검색.\n하위 명령 하나의 형식: hive knowledge <하위 명령> --help\n주요 하위 명령: remember, retrieve, query, list, read, lint,\nexport, import, transfer, scan, graph, vector",
            "Saves and finds project and user knowledge as canonical Markdown.\nOne subcommand's syntax: hive knowledge <subcommand> --help\nMain subcommands: remember, retrieve, query, list, read, lint,\nexport, import, transfer, scan, graph, vector"),
    required: &[
        ("--target <폴더>", "대상 프로젝트 폴더 (대부분)", "Project folder (most subcommands)"),
        ("--user-root <폴더>", "사용자 폴더 (remember, retrieve 등)", "User folder (remember, retrieve, ...)"),
    ],
    optional: &[
        ("--text <검색어>", "검색어 (query)", "Search text (query)"),
        ("--query <검색어>", "검색어 (retrieve)", "Search text (retrieve)"),
        ("--limit <1..100>", "결과 수 제한", "Result limit"),
        ("--output json", "기계용 JSON 출력", "Machine-readable JSON output"),
    ],
    examples: &["hive knowledge lint --target .",
                "hive knowledge query --target . --text build",
                "hive knowledge retrieve --user-root <user-root> --target . --query build"],
};

#[rustfmt::skip]
const USAGE_HELP: Detailed = Detailed {
    title: "hive usage",
    summary: ("사용량 확인과 사용량 보호", "check usage and the usage guard"),
    about: ("AI 도구 사용량을 확인하고, 남은 양이 기준보다 적으면 작업을 멈춤.\n하위 명령: show, status, threshold, session, monitor, enforce,\ncheck, probe-native, capture, summarize-tokens, fallback-install",
            "Checks AI tool usage and stops work when too little remains.\nSubcommands: show, status, threshold, session, monitor, enforce,\ncheck, probe-native, capture, summarize-tokens, fallback-install"),
    required: &[
        ("--user-root <폴더>", "사용자 폴더 (show, threshold, monitor)", "User folder (show, threshold, monitor)"),
        ("--target <폴더>", "프로젝트 폴더 (status, enforce, session)", "Project folder (status, enforce, session)"),
        ("--session-id <id>", "현재 세션 id (status, enforce, session)", "Current session id (same)"),
        ("--process-id <번호>", "현재 프로세스 번호 (위와 같음)", "Current process id (same)"),
    ],
    optional: &[
        ("--host <호스트>", "codex, claude, antigravity 중 하나", "One of codex, claude, antigravity"),
        ("--remaining-percent <0..99>", "모든 보호 한도의 남은 양 기준 % (threshold)", "Threshold %, all guarded limits (threshold)"),
        ("--output json", "기계용 JSON 출력", "Machine-readable JSON output"),
    ],
    examples: &["hive usage threshold --user-root <user-root> --remaining-percent 20",
                "hive usage monitor status --user-root <user-root>"],
};

#[rustfmt::skip]
const PROJECT_HELP: Detailed = Detailed {
    title: "hive project upgrade",
    summary: ("프로젝트 Hive 지침 갱신", "upgrade a project's Hive harness"),
    about: ("설치한 프로젝트 하네스를 현재 Hive 버전에 맞게 갱신.\n사용자가 고친 내용은 보존. 먼저 --scan 또는 --dry-run으로 확인.",
            "Upgrades an installed project harness to this Hive version.\nYour edits are kept. Check with --scan or --dry-run first."),
    required: &[
        ("--target <폴더>", "갱신할 프로젝트 폴더", "Project folder to upgrade"),
        ("--scan|--dry-run|--apply", "검사 | 미리 보기 | 적용", "Scan | preview | apply"),
        ("--validate|--recover", "또는 확인 | 복구 중 하나", "Or validate | recover"),
    ],
    optional: &[
        ("--user-root <폴더>", "사용자 폴더 (기본: 홈 폴더)", "User folder (default: home)"),
        ("--skill-merge-inputs <skill>", "Skill 병합 입력 보기", "Show Skill merge inputs"),
        ("--skill-merges <json>", "Skill 병합 결과 파일", "Skill merge result file"),
        ("--approve-skill-merge <digest>", "확인한 병합 승인", "Approve the reviewed merge"),
        ("--output json", "기계용 JSON 출력", "Machine-readable JSON output"),
    ],
    examples: &["hive project upgrade --target . --scan",
                "hive project upgrade --target . --dry-run",
                "hive project upgrade --target . --apply"],
};

fn push_rows(text: &mut String, rows: &[Row], ko: bool) {
    for (option, ko_text, en_text) in rows {
        let description = if ko { ko_text } else { en_text };
        if width(option) > 28 {
            writeln!(text, "  {option}\n  {} {description}", pad("", 28))
                .expect("formatting a String");
        } else {
            writeln!(text, "  {} {description}", pad(option, 28)).expect("formatting a String");
        }
    }
}

fn detailed_text(help: &Detailed, syntax: &[String], ko: bool) -> String {
    let (summary, about) = if ko {
        (help.summary.0, help.about.0)
    } else {
        (help.summary.1, help.about.1)
    };
    let mut text = format!("{}: {summary}\n\n{about}\n\n", help.title);
    text.push_str(if ko {
        "필수 옵션\n"
    } else {
        "Required options\n"
    });
    if help.required.is_empty() {
        text.push_str(if ko { "  없음\n" } else { "  None\n" });
    }
    push_rows(&mut text, help.required, ko);
    text.push_str(if ko {
        "\n선택 옵션\n"
    } else {
        "\nOptional\n"
    });
    push_rows(&mut text, help.optional, ko);
    text.push_str(if ko { "\n예시\n" } else { "\nExamples\n" });
    for example in help.examples {
        writeln!(text, "  {example}").expect("formatting a String");
    }
    text.push_str(if ko {
        "\n전체 형식\n"
    } else {
        "\nFull syntax\n"
    });
    for line in syntax {
        text.push_str(line);
        text.push('\n');
    }
    wrap(&text)
}

/// Syntax lines (`hive ...`) from a usage text, with no prose.
fn syntax_lines(text: &str, prefix: &str) -> Vec<String> {
    text.lines()
        .filter(|line| {
            let line = line.trim_start();
            (line == prefix || line.starts_with(&format!("{prefix} ")))
                && !line.trim_end().ends_with("--help")
        })
        .map(|line| format!("    {}", line.trim()))
        .collect()
}

fn update_syntax() -> Vec<String> {
    let mut lines = syntax_lines(crate::USAGE, "hive update");
    lines.extend(syntax_lines(crate::update::UPDATE_USAGE, "hive update"));
    lines.push(
        "    hive update --target <dir> --bundle <release-dir> --apply --exact-major-target <X.Y.Z> --major-confirmation <confirmation.json> [--output json]"
            .to_owned(),
    );
    lines
}

fn setup_syntax(ko: bool) -> Vec<String> {
    let mut lines = syntax_lines(crate::USAGE, "hive setup");
    lines.push(String::new());
    lines.push(
        if ko {
            "전역 설정의 전체 형식: hive setup --scope user --help"
        } else {
            "Global user setup syntax: hive setup --scope user --help"
        }
        .to_owned(),
    );
    lines.push(String::new());
    lines.push("MODES:".to_owned());
    let modes = crate::SETUP_USAGE.split("MODES:").nth(1).unwrap_or("");
    // Table rows stay as they are; a hard-wrapped prose paragraph becomes one line to rewrap.
    let mut paragraph = String::new();
    for line in modes.lines().skip(1) {
        if line.starts_with(' ') || line.is_empty() {
            lines.push(line.to_owned());
        } else {
            if !paragraph.is_empty() {
                paragraph.push(' ');
            }
            paragraph.push_str(line);
        }
    }
    if !paragraph.is_empty() {
        lines.push(paragraph);
    }
    lines
}

fn knowledge_usage() -> String {
    let usage = crate::knowledge::KNOWLEDGE_USAGE.to_owned();
    #[cfg(feature = "notion-preview")]
    let usage = usage + crate::knowledge::NOTION_KNOWLEDGE_USAGE;
    usage
}

/// Help for one knowledge subcommand, or `None` when the subcommand has its own help.
fn knowledge_subcommand_help(sub: &str, ko: bool) -> Option<String> {
    if sub == "vector" {
        return None;
    }
    let lines = syntax_lines(&knowledge_usage(), &format!("hive knowledge {sub}"));
    if lines.is_empty() {
        return None;
    }
    let mut text = format!("hive knowledge {sub}\n\n");
    text.push_str(if ko { "형식\n" } else { "Syntax\n" });
    for line in lines {
        text.push_str(&line);
        text.push('\n');
    }
    if let Some(example) = example_for("knowledge", Some(sub))
        .filter(|example| example.starts_with(&format!("hive knowledge {sub} ")))
    {
        text.push_str(if ko { "\n예시\n" } else { "\nExample\n" });
        writeln!(text, "  {example}").expect("formatting a String");
    }
    text.push_str(if ko {
        "\n전체 지식 명령: hive knowledge --help\n"
    } else {
        "\nAll knowledge commands: hive knowledge --help\n"
    });
    Some(wrap(&text))
}

fn asks_help(arguments: &[String]) -> bool {
    arguments
        .iter()
        .any(|argument| argument == "--help" || argument == "-h")
}

fn only_help(arguments: &[String]) -> bool {
    matches!(arguments, [argument] if argument == "--help" || argument == "-h")
}

/// Detailed help for the main commands. Other commands keep their existing help paths.
pub(crate) fn help_request(arguments: &[String], language: InterfaceLanguage) -> Option<String> {
    let ko = language == InterfaceLanguage::Ko;
    let rest = arguments.get(1..)?;
    match arguments.first()?.as_str() {
        "install" if asks_help(rest) => Some(detailed_text(
            &INSTALL_HELP,
            &syntax_lines(crate::USAGE, "hive install"),
            ko,
        )),
        "setup" if only_help(rest) => Some(detailed_text(&SETUP_HELP, &setup_syntax(ko), ko)),
        "update" if only_help(rest) => Some(detailed_text(&UPDATE_HELP, &update_syntax(), ko)),
        "knowledge" if only_help(rest) => Some(detailed_text(
            &KNOWLEDGE_HELP,
            &syntax_lines(&knowledge_usage(), "hive knowledge"),
            ko,
        )),
        "knowledge" => match rest {
            [sub, help] if help == "--help" || help == "-h" => knowledge_subcommand_help(sub, ko),
            _ => None,
        },
        "usage" if rest.is_empty() || only_help(rest) => Some(detailed_text(
            &USAGE_HELP,
            &syntax_lines(crate::USAGE, "hive usage"),
            ko,
        )),
        "project" if asks_help(rest) => Some(detailed_text(
            &PROJECT_HELP,
            &syntax_lines(crate::USAGE, "hive project"),
            ko,
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
