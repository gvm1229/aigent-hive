//! OV160-005 containment tests with fake consumers only: no host, worker or provider runs,
//! no keystrokes and no process control.
use super::*;
use std::path::PathBuf;

/// Athena reviews on Claude (no official cancel); Hephaistos and Artemis stay on Codex.
const ROSTER: &str = "[gods]\nathena = 'hive-independent-judge'\n\n[roles.hive-independent-judge]\nprovider = 'claude'\nmodel = 'claude-fable-5-2'\neffort = 'high'\n";

struct Fixture {
    root: tempfile::TempDir,
    target: PathBuf,
    home: PathBuf,
}

fn fixture() -> Fixture {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("consumer");
    let home = root.path().join("home");
    std::fs::create_dir_all(target.join(".hive/config")).unwrap();
    std::fs::create_dir_all(home.join(".hive")).unwrap();
    std::fs::write(home.join(".hive/roster.toml"), ROSTER).unwrap();
    std::fs::write(root.path().join("brief.md"), "Implement and summarize.").unwrap();
    Fixture { root, target, home }
}

impl Fixture {
    fn scoped(&self, arguments: &[&str]) -> Vec<String> {
        let mut all: Vec<String> = arguments.iter().map(|a| (*a).to_owned()).collect();
        all.extend([
            "--target".into(),
            self.target.to_str().unwrap().into(),
            "--user-root".into(),
            self.home.to_str().unwrap().into(),
        ]);
        all
    }

    fn validated(result: ActionResult) -> Value {
        let value = serde_json::to_value(result).unwrap();
        hive_core::validate_json_schema(
            include_str!("../../../../schemas/action-result.schema.json"),
            &value,
            "containment action result",
        )
        .unwrap();
        value
    }

    fn run(&self, arguments: &[&str]) -> Value {
        Self::validated(
            execute(&self.scoped(arguments)).unwrap_or_else(|error| failure(error, None)),
        )
    }

    fn stop(&self, arguments: &[&str]) -> Value {
        Self::validated(stop::test_execute(&self.scoped(arguments)))
    }

    fn file(&self, name: &str, text: &str) -> String {
        let path = self.root.path().join(name);
        std::fs::write(&path, text).unwrap();
        path.to_str().unwrap().to_owned()
    }

    fn harness(&self, text: &str) {
        std::fs::write(
            self.target.join(".hive/config/harness.toml"),
            format!("schema_version = 1\nharness_version = \"0.16.0\"\n{text}"),
        )
        .unwrap();
    }

    fn send(&self, body: &str, to: &str, task: &str) -> Value {
        let file = self.file("body.txt", body);
        self.run(&[
            "send", "--to", to, "--from", "athena", "--file", &file, "--task", task, "--now",
            "1000",
        ])
    }

    fn sent_id(&self, body: &str, to: &str, task: &str) -> String {
        let sent = self.send(body, to, task);
        assert_eq!(sent["code"], "hive.msg-queued", "{sent}");
        sent["data"]["message"]["id"].as_str().unwrap().to_owned()
    }

    fn claim(&self, id: &str, role: &str, extra: &[&str]) -> Value {
        let mut arguments = vec!["claim", "--id", id, "--role", role, "--now", "1001"];
        arguments.extend_from_slice(extra);
        self.run(&arguments)
    }

    fn ack(&self, id: &str, text: &str, now: &str) -> Value {
        let result = self.file("result.txt", text);
        self.run(&["ack", "--id", id, "--result", &result, "--now", now])
    }

    fn start(&self, extra: &[&str]) -> Value {
        let brief = self.root.path().join("brief.md");
        let mut arguments = vec![
            "flow",
            "start",
            "--builder",
            "hephaistos",
            "--reviewer",
            "athena",
            "--task",
            "task-5",
            "--file",
            brief.to_str().unwrap(),
            "--now",
            "1000",
        ];
        arguments.extend_from_slice(extra);
        self.run(&arguments)
    }

    /// Every file under `.hive/mediation/inbox`, the envelope count proof.
    fn envelopes(&self) -> usize {
        count(&self.target.join(".hive/mediation/inbox"), "json")
    }
}

fn count(path: &std::path::Path, extension: &str) -> usize {
    std::fs::read_dir(path).map_or(0, |entries| {
        entries
            .map(|entry| {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    count(&entry.path(), extension)
                } else {
                    usize::from(entry.path().extension().is_some_and(|e| e == extension))
                }
            })
            .sum()
    })
}

fn flow_id(value: &Value) -> String {
    value["data"]["flow"]["flow_id"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn step_id(value: &Value, index: usize) -> String {
    value["data"]["flow"]["steps"][index]["message_id"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn containment_every_message_carries_a_least_privilege_policy() {
    let f = fixture();
    let id = f.sent_id("review", "artemis", "task-1");
    assert!(f
        .target
        .join(hive_core::containment::policy_path(&id))
        .is_file());
    let claimed = f.claim(&id, "artemis", &[]);
    assert_eq!(claimed["code"], "hive.msg-claimed", "{claimed}");
    let policy = &claimed["data"]["message"]["worker_policy"];
    assert_eq!(policy["present"], true, "{policy}");
    assert_eq!(policy["enforced_by_hive"], false);
    let policy = &policy["policy"];
    assert_eq!(policy["sandbox"], "read-only");
    assert_eq!(policy["network"], "none");
    assert_eq!(policy["git_push"], "deny");
    assert_eq!(policy["writable_roots"], json!([]));
    assert_eq!(policy["git_env"]["GIT_CONFIG_VALUE_0"], "never");
    assert_eq!(policy["hive_launches_worker"], false);
    let builder = f.sent_id("build", "hephaistos", "task-1");
    let view = f.run(&["status", "--id", &builder, "--now", "1001"]);
    let policy = &view["data"]["message"]["worker_policy"]["policy"];
    assert_eq!(policy["sandbox"], "workspace-write");
    assert_eq!(policy["writable_roots"], json!(["{target}"]));
    // A tampered policy is reported invalid, never silently trusted.
    let path = f.target.join(hive_core::containment::policy_path(&builder));
    let tampered = std::fs::read_to_string(&path)
        .unwrap()
        .replace("\"none\"", "\"enabled\"");
    std::fs::write(&path, tampered).unwrap();
    let view = f.run(&["status", "--id", &builder, "--now", "1001"]);
    assert!(view["data"]["message"]["worker_policy"]["invalid"].is_string());
}

#[test]
fn containment_max_workers_limits_claims_and_time_ceiling_expires() {
    let f = fixture();
    f.harness("max_workers = 1\nmax_worker_seconds = 60\n");
    let first = f.sent_id("one", "artemis", "task-1");
    let second = f.sent_id("two", "artemis", "task-2");
    let refused = f.claim(&first, "artemis", &["--lease", "120"]);
    assert_eq!(refused["code"], "hive.msg-worker-time-limit", "{refused}");
    assert_eq!(refused["exit_code"], 3);
    let claimed = f.claim(&first, "artemis", &[]);
    assert_eq!(claimed["code"], "hive.msg-claimed", "{claimed}");
    // Default lease is capped by the ceiling.
    assert_eq!(claimed["data"]["lease_until"], 1061);
    assert_eq!(claimed["data"]["limits"]["max_workers"], 1);
    let limited = f.claim(&second, "artemis", &[]);
    assert_eq!(limited["code"], "hive.msg-worker-limit", "{limited}");
    assert!(!f.target.join(mediation::claim_path(&second)).exists());
    // The ceiling ends the claim: dispatch-uncertain, never re-sent, and the slot frees up.
    let expired = f.run(&["status", "--id", &first, "--now", "1061"]);
    assert_eq!(expired["code"], "hive.msg-dispatch-uncertain", "{expired}");
    let freed = f.run(&[
        "claim", "--id", &second, "--role", "artemis", "--now", "1062",
    ]);
    assert_eq!(freed["code"], "hive.msg-claimed", "{freed}");
}

#[test]
fn containment_max_panes_value_is_read_and_a_conflict_is_refused() {
    let f = fixture();
    f.harness("max_panes = 1\n");
    let first = f.sent_id("one", "artemis", "task-1");
    let second = f.sent_id("two", "artemis", "task-2");
    let claimed = f.claim(&first, "artemis", &[]);
    assert_eq!(claimed["data"]["limits"]["source"], "legacy-max-panes");
    let limited = f.claim(&second, "artemis", &[]);
    assert_eq!(limited["code"], "hive.msg-worker-limit", "{limited}");
    f.harness("max_panes = 1\nmax_workers = 1\n");
    let same = f.run(&[
        "claim", "--id", &second, "--role", "artemis", "--now", "1001",
    ]);
    assert_eq!(same["code"], "hive.msg-worker-limit", "{same}");
    f.harness("max_panes = 1\nmax_workers = 3\n");
    let before = std::fs::read(f.target.join(".hive/config/harness.toml")).unwrap();
    let conflict = f.claim(&second, "artemis", &[]);
    assert_eq!(conflict["code"], "hive.msg-limit-conflict", "{conflict}");
    assert!(conflict["message"].as_str().unwrap().contains("conflicts"));
    assert_eq!(
        std::fs::read(f.target.join(".hive/config/harness.toml")).unwrap(),
        before
    );
}

#[test]
fn containment_changed_scorecard_quarantines_the_flow() {
    let f = fixture();
    let inside = f.target.join("score.md");
    std::fs::write(&inside, "inside").unwrap();
    let refused = f.start(&["--check", inside.to_str().unwrap()]);
    assert_eq!(
        refused["code"], "hive.msg-check-inside-worktree",
        "{refused}"
    );
    assert_eq!(f.envelopes(), 0);
    // A separate worktree: the check beside the project is outside it and stays read-only.
    let worktree = f.root.path().join("task-worktree");
    std::fs::create_dir_all(&worktree).unwrap();
    let scorecard = f.file("scorecard.md", "pass if tests are green");
    let started = f.start(&[
        "--check",
        &scorecard,
        "--worktree",
        worktree.to_str().unwrap(),
    ]);
    assert_eq!(started["code"], "hive.msg-flow-started", "{started}");
    let flow = flow_id(&started);
    let checks = &started["data"]["flow"]["checks"];
    assert_eq!(checks["drift"], false, "{checks}");
    assert_eq!(checks["read_only_for_worker"], true);
    let pinned = checks["pinned"][0]["digest"].as_str().unwrap().to_owned();
    assert_eq!(pinned, hive_core::sha256_digest(b"pass if tests are green"));
    let builder = step_id(&started, 0);
    let claimed = f.claim(&builder, "hephaistos", &["--lease", "60"]);
    assert_eq!(claimed["code"], "hive.msg-claimed", "{claimed}");
    std::fs::write(&scorecard, "pass always").unwrap();
    let acked = f.ack(&builder, "built", "1002");
    assert_eq!(acked["code"], "hive.msg-acknowledged", "{acked}");
    assert_eq!(
        acked["data"]["flow_error"]["code"], "hive.msg-flow-check-drift",
        "{acked}"
    );
    assert_eq!(f.envelopes(), 1, "no reviewer step after the drift");
    let status = f.run(&["flow", "status", "--flow", &flow, "--now", "1003"]);
    assert_eq!(status["code"], "hive.msg-flow-stopped", "{status}");
    assert_eq!(status["data"]["flow"]["state"], "stopped");
    assert_eq!(
        status["data"]["flow"]["checks"]["drift_record"]["pinned_digest"],
        pinned
    );
    // Restoring the bytes does not resume: the recorded drift stays.
    std::fs::write(&scorecard, "pass if tests are green").unwrap();
    let advance = f.run(&["flow", "advance", "--flow", &flow, "--now", "1004"]);
    assert_eq!(advance["code"], "hive.msg-flow-check-drift", "{advance}");
    assert_eq!(f.envelopes(), 1);
}

#[test]
#[allow(clippy::too_many_lines)]
fn containment_stop_creates_zero_new_envelopes_and_waits_for_host_receipts() {
    let f = fixture();
    let started = f.start(&[]);
    let flow = flow_id(&started);
    let builder = step_id(&started, 0);
    let codex_work = f.sent_id("research", "artemis", "task-1");
    let claude_work = f.sent_id("judge", "athena", "task-2");
    let queued = f.sent_id("later", "artemis", "task-3");
    for (id, role) in [
        (&builder, "hephaistos"),
        (&codex_work, "artemis"),
        (&claude_work, "athena"),
    ] {
        let claimed = f.claim(id, role, &["--lease", "60"]);
        assert_eq!(claimed["code"], "hive.msg-claimed", "{claimed}");
    }
    let stopped = f.stop(&["--reason", "user stop", "--now", "1010"]);
    assert_eq!(stopped["code"], "hive.stop-requested", "{stopped}");
    let view = &stopped["data"]["stop"];
    assert_eq!(view["forced"], false);
    assert_eq!(view["workers_interrupted"], 0);
    assert_eq!(view["keystrokes_sent"], 0);
    let in_progress = view["in_progress"].as_array().unwrap();
    assert_eq!(in_progress.len(), 3, "{view}");
    let support = |id: &str| {
        in_progress
            .iter()
            .find(|entry| entry["message_id"] == id)
            .unwrap()
            .clone()
    };
    assert_eq!(support(&claude_work)["host_cancel"], "unsupported");
    assert_eq!(support(&claude_work)["state"], "cancel-requested");
    assert_eq!(support(&codex_work)["host_cancel"], "unverified");
    assert_eq!(view["unsupported_cancel"], 1);
    assert!(stopped["message"].as_str().unwrap().contains("unsupported"));
    // Human output says "unsupported" plainly and claims no forced cancel.
    let human =
        crate::human_output::render(&stopped, crate::user_setup::InterfaceLanguage::Ko, "stop");
    assert!(human.contains("공식 취소 미지원"), "{human}");
    assert!(human.contains("강제 취소 없음"), "{human}");
    let again = f.stop(&["--now", "1011"]);
    assert_eq!(again["code"], "hive.stop-already-active");
    assert_eq!(count(&f.target.join(".hive/mediation/stops"), "json"), 1);

    // Zero new envelopes: send, flow start, flow advance, requeue, claim.
    let before = f.envelopes();
    let sent = f.send("new work", "artemis", "task-9");
    assert_eq!(sent["code"], "hive.msg-stopped", "{sent}");
    assert!(!f
        .target
        .join(".hive/mediation/keys")
        .read_dir()
        .unwrap()
        .any(|e| {
            let name = e.unwrap().file_name();
            std::fs::read_to_string(f.target.join(".hive/mediation/keys").join(name))
                .unwrap()
                .contains("task-9")
        }));
    let brief = f.file("other.md", "another brief");
    let other = f.run(&[
        "flow",
        "start",
        "--builder",
        "hephaistos",
        "--reviewer",
        "athena",
        "--task",
        "task-8",
        "--file",
        &brief,
        "--now",
        "1012",
    ]);
    assert_eq!(other["code"], "hive.msg-stopped", "{other}");
    let claim_queued = f.run(&[
        "claim", "--id", &queued, "--role", "artemis", "--now", "1012",
    ]);
    assert_eq!(claim_queued["code"], "hive.msg-stopped", "{claim_queued}");
    // In-progress builder result after the stop: quarantined, no reviewer step.
    let late = f.ack(&builder, "built after stop", "1013");
    assert_eq!(late["code"], "hive.msg-result-quarantined", "{late}");
    let advanced = f.run(&["flow", "advance", "--flow", &flow, "--now", "1014"]);
    assert_ne!(advanced["code"], "hive.msg-flow-advanced");
    // The lease ends: the builder is quarantined by its late result; requeue stays refused.
    let status = f.run(&["flow", "status", "--flow", &flow, "--now", "1100"]);
    assert_eq!(
        status["data"]["flow"]["stopped_by"],
        stopped["data"]["stop"]["record"]["stop_id"]
    );
    assert_eq!(f.envelopes(), before, "zero new envelopes after the stop");

    // Unsupported host: no cancel can be recorded; nothing claims a forced cancel.
    let receipt_for = |id: &str, host: &str| {
        let message = f.run(&["status", "--id", id, "--now", "1015"]);
        let message = &message["data"]["message"];
        assert_eq!(message["state"], "cancel-requested", "{message}");
        f.file(
            "cancel.json",
            &json!({"schema_version": 1, "message_id": id,
                "envelope_digest": message["envelope_digest"],
                "claim_digest": message["claim_digest"],
                "stop_id": message["stopped_by"], "host": host,
                "method": "host-official-cancel", "cancelled_at": 1015, "forced": false,
                "trust": "untrusted-data"})
            .to_string(),
        )
    };
    let claude_receipt = receipt_for(&claude_work, "claude");
    let unsupported = f.run(&[
        "cancel-ack",
        "--id",
        &claude_work,
        "--receipt",
        &claude_receipt,
        "--now",
        "1015",
    ]);
    assert_eq!(
        unsupported["code"], "hive.msg-cancel-unsupported",
        "{unsupported}"
    );
    assert!(!f
        .target
        .join(hive_core::containment::cancel_receipt_path(&claude_work))
        .exists());
    // A forced cancel is never recorded.
    let forced = f.file(
        "forced.json",
        &std::fs::read_to_string(receipt_for(&codex_work, "codex"))
            .unwrap()
            .replace("\"forced\":false", "\"forced\":true"),
    );
    let refused = f.run(&[
        "cancel-ack",
        "--id",
        &codex_work,
        "--receipt",
        &forced,
        "--now",
        "1015",
    ]);
    assert_eq!(refused["code"], "hive.msg-cancel-refused", "{refused}");
    // The host's own official cancel receipt moves the work to cancelled.
    let codex_receipt = receipt_for(&codex_work, "codex");
    let cancelled = f.run(&[
        "cancel-ack",
        "--id",
        &codex_work,
        "--receipt",
        &codex_receipt,
        "--now",
        "1016",
    ]);
    assert_eq!(cancelled["code"], "hive.msg-cancel-recorded", "{cancelled}");
    assert_eq!(cancelled["data"]["message"]["state"], "cancelled");
    assert_eq!(cancelled["data"]["forced"], false);
    assert_eq!(cancelled["data"]["interrupted_by_hive"], false);
    // A late result after the cancel is quarantined and never delivered.
    let late = f.ack(&codex_work, "late research", "1017");
    assert_eq!(late["code"], "hive.msg-result-quarantined", "{late}");
    let status = f.run(&["status", "--id", &codex_work, "--now", "1018"]);
    assert_eq!(status["data"]["message"]["state"], "cancelled");
    assert!(status["data"]["message"]["result"].is_null());
    // Without a receipt the unsupported work ends by its lease, never as "cancelled".
    let ended = f.run(&["status", "--id", &claude_work, "--now", "1100"]);
    assert_eq!(ended["data"]["message"]["state"], "dispatch-uncertain");

    // Explicit resume only with the exact stop digest.
    let preview = f.stop(&["clear", "--now", "1101"]);
    assert_eq!(preview["code"], "hive.stop-clear-preview", "{preview}");
    let wrong = f.stop(&[
        "clear",
        "--confirm",
        &hive_core::sha256_digest(b"x"),
        "--now",
        "1101",
    ]);
    assert_eq!(wrong["code"], "hive.stop-clear-refused");
    let digest = preview["data"]["confirm"].as_str().unwrap().to_owned();
    let cleared = f.stop(&["clear", "--confirm", &digest, "--now", "1102"]);
    assert_eq!(cleared["code"], "hive.stop-cleared", "{cleared}");
    assert_eq!(cleared["data"]["redispatched"], false);
    assert_eq!(f.envelopes(), before, "clearing re-sends nothing");
    let resumed = f.send("new work", "artemis", "task-9");
    assert_eq!(resumed["code"], "hive.msg-queued", "{resumed}");
    let status = f.stop(&["status", "--now", "1103"]);
    assert_eq!(status["data"]["stopped"], false);
}

#[test]
fn containment_task_and_flow_stops_cover_only_their_scope() {
    let f = fixture();
    let started = f.start(&[]);
    let flow = flow_id(&started);
    let stopped = f.stop(&["--flow", &flow, "--now", "1001"]);
    assert_eq!(stopped["code"], "hive.stop-requested", "{stopped}");
    let builder = step_id(&started, 0);
    let refused = f.claim(&builder, "hephaistos", &[]);
    assert_eq!(refused["code"], "hive.msg-stopped", "{refused}");
    // Other work keeps flowing.
    let other = f.send("unrelated", "artemis", "task-1");
    assert_eq!(other["code"], "hive.msg-queued", "{other}");
    let task = f.stop(&["--task", "task-1", "--now", "1002"]);
    assert_eq!(task["code"], "hive.stop-requested");
    let refused = f.send("more", "artemis", "task-1");
    assert_eq!(refused["code"], "hive.msg-stopped", "{refused}");
    let allowed = f.send("elsewhere", "artemis", "task-2");
    assert_eq!(allowed["code"], "hive.msg-queued", "{allowed}");
    let missing = f.stop(&["--flow", "flow-missing", "--now", "1003"]);
    assert_eq!(missing["code"], "hive.msg-flow-not-found");
    // An unreadable stop record fails closed for the whole project.
    std::fs::write(
        f.target.join(".hive/mediation/stops/stop-broken.json"),
        "{}",
    )
    .unwrap();
    let closed = f.send("anything", "artemis", "task-2b");
    assert_eq!(closed["code"], "hive.msg-stopped", "{closed}");
}
