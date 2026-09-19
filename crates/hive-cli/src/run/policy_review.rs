//! Explicit review of bounded, run-bound policy evidence; never automatic policy learning.

use super::{as_set, parse_options, required, run_path, AdapterError, PinnedTarget};
use crate::{emit_action_result, ActionResult};
use hive_core::policy::{evaluate, valid_digest, Decision, Evaluation, FailureClass};
use hive_core::run::{Host, RunPlan, RunStatusDocument};
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::ExitCode;

const LIMIT: usize = 256 * 1024;
const HEADER: &str = "# Policy review candidates\n\n```json\n";
const FOOTER: &str = "\n```\n";
const RULES: &[&str] = &[
    "source-branch",
    "commit-concern",
    "fresh-verification",
    "active-plan",
    "task-closure",
    "source-consumer",
    "hive-owned-state",
    "concurrent-edit",
    "update-recovery",
    "historical-bytes",
    "canonical-freshness",
    "knowledge-scope",
    "reviewed-memory",
    "usage-preflight",
    "exact-authority",
    "release-authority",
    "host-owned-model",
    "language-quality",
    "intent-routing",
    "cancel-resume",
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ReviewState {
    Pending,
    Accepted,
    Revised,
    Rejected,
    Cancelled,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Candidate {
    id: String,
    rule: String,
    policy_digest: String,
    proposed_class: FailureClass,
    revised_class: Option<FailureClass>,
    supporting_evaluations: BTreeSet<String>,
    counter_evaluations: BTreeSet<String>,
    state: ReviewState,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ReviewBook {
    schema_version: u32,
    scope_digest: String,
    host: Host,
    host_version_digest: String,
    operating_system: String,
    candidates: Vec<Candidate>,
}

fn verification(message: &str) -> AdapterError {
    AdapterError::Verification(message.to_owned())
}

fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, AdapterError> {
    serde_json_canonicalizer::to_vec(value).map_err(|_| verification("cannot encode review data"))
}

fn candidate_id(rule: &str, policy: &str, class: FailureClass) -> Result<String, AdapterError> {
    Ok(sha256_digest(&encode(&(rule, policy, class))?))
}

fn class(value: &str) -> Result<FailureClass, AdapterError> {
    serde_json::from_value(json!(value)).map_err(|_| verification("unknown failure class"))
}

impl ReviewBook {
    fn validate(&self) -> Result<(), AdapterError> {
        if self.schema_version != 1
            || !valid_digest(&self.scope_digest)
            || !valid_digest(&self.host_version_digest)
            || self.candidates.len() > 128
            || !["windows", "linux", "macos"].contains(&self.operating_system.as_str())
        {
            return Err(verification("invalid review book binding or bounds"));
        }
        let mut ids = BTreeSet::new();
        for candidate in &self.candidates {
            if !RULES.contains(&candidate.rule.as_str())
                || !valid_digest(&candidate.policy_digest)
                || candidate.id
                    != candidate_id(
                        &candidate.rule,
                        &candidate.policy_digest,
                        candidate.proposed_class,
                    )?
                || !ids.insert(&candidate.id)
                || candidate.supporting_evaluations.is_empty()
                || candidate.supporting_evaluations.len() + candidate.counter_evaluations.len() > 32
                || !candidate
                    .supporting_evaluations
                    .is_disjoint(&candidate.counter_evaluations)
                || !candidate
                    .supporting_evaluations
                    .iter()
                    .chain(&candidate.counter_evaluations)
                    .all(|digest| valid_digest(digest))
                || (candidate.state == ReviewState::Revised) != candidate.revised_class.is_some()
            {
                return Err(verification("invalid bounded review candidate"));
            }
        }
        Ok(())
    }

    fn bytes(&self) -> Result<Vec<u8>, AdapterError> {
        self.validate()?;
        let mut bytes = HEADER.as_bytes().to_vec();
        bytes.extend(encode(self)?);
        bytes.extend(FOOTER.as_bytes());
        if bytes.len() > LIMIT {
            return Err(verification("review book exceeds size bound"));
        }
        Ok(bytes)
    }

    fn observe(
        &mut self,
        evaluation: &Evaluation,
        digest: &str,
        rule: &str,
        proposed: FailureClass,
    ) -> Result<(), AdapterError> {
        let id = candidate_id(rule, &evaluation.binding.policy_digest, proposed)?;
        let result = evaluation
            .results
            .iter()
            .find(|result| result.rule_id == rule)
            .ok_or_else(|| verification("selected rule has no observed checker result"))?;
        let existing = self
            .candidates
            .iter_mut()
            .find(|candidate| candidate.id == id);
        let candidate = if let Some(candidate) = existing {
            candidate
        } else {
            if result.decision == Decision::Allow {
                return Err(verification(
                    "allowing evidence cannot create a failure candidate",
                ));
            }
            self.candidates.push(Candidate {
                id,
                rule: rule.to_owned(),
                policy_digest: evaluation.binding.policy_digest.clone(),
                proposed_class: proposed,
                revised_class: None,
                supporting_evaluations: BTreeSet::new(),
                counter_evaluations: BTreeSet::new(),
                state: ReviewState::Pending,
            });
            self.candidates.last_mut().expect("just inserted")
        };
        let evidence = if result.decision == Decision::Allow {
            &mut candidate.counter_evaluations
        } else {
            &mut candidate.supporting_evaluations
        };
        if evidence.insert(digest.to_owned()) {
            candidate.state = ReviewState::Pending;
            candidate.revised_class = None;
        }
        self.candidates.sort_by(|a, b| a.id.cmp(&b.id));
        self.validate()
    }
}

pub(super) fn run(args: &[String]) -> ExitCode {
    if args == ["--help"] {
        println!("hive run policy-review preview|add|list|accept|revise|reject|cancel --target <dir> --run <id> --output json\npreview/add: --evaluation <run-relative-file> --rule <registered-rule> --class <failure-class>\nadd: --confirm <preview-digest>\naccept/revise/reject/cancel: --candidate <digest> --confirm <book-digest> [--class <revised-class>]\nExplicit bounded review only; acceptance never grants file mutation authority.");
        return ExitCode::SUCCESS;
    }
    let result = execute(args)
        .unwrap_or_else(|error| super::failure_result("ReviewPolicyCandidate", &error));
    emit_action_result(&result)
}

#[allow(clippy::too_many_lines)]
fn execute(args: &[String]) -> Result<ActionResult, AdapterError> {
    let verb = args
        .first()
        .map(String::as_str)
        .ok_or_else(|| verification("missing review action"))?;
    let extra: &[&str] = match verb {
        "preview" => &["--evaluation", "--rule", "--class"],
        "add" => &["--evaluation", "--rule", "--class", "--confirm"],
        "list" => &[],
        "accept" | "reject" | "cancel" => &["--candidate", "--confirm"],
        "revise" => &["--candidate", "--confirm", "--class"],
        _ => return Err(verification("unknown review action")),
    };
    let allowed = [vec!["--target", "--run"], extra.to_vec()].concat();
    let options = parse_options(&args[1..], &allowed)?;
    let target = PinnedTarget::open(Path::new(required(&options, "--target")?))?;
    let run_id = required(&options, "--run")?;
    // A single run directory only; no nested IDs or traversal into another run.
    if run_id.is_empty()
        || run_id.len() > 128
        || !run_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
    {
        return Err(verification("invalid review run id"));
    }
    let plan_path = run_path(run_id, "PLAN.md")?;
    let status_path = run_path(run_id, "STATUS.md")?;
    let plan_bytes = target.read_required(&plan_path, LIMIT)?;
    let status_bytes = target.read_required(&status_path, LIMIT)?;
    let plan = RunPlan::parse_markdown(&plan_bytes).map_err(super::core_verification)?;
    let status =
        RunStatusDocument::parse_markdown(&status_bytes).map_err(super::core_verification)?;
    status
        .validate_checkpoint()
        .map_err(super::core_verification)?;
    if status.status().run_id != run_id
        || as_set(plan.criteria()) != as_set(&status.status().required_criteria)
    {
        return Err(verification("review run plan and status disagree"));
    }
    let owner = status.owner_binding().map_err(super::core_verification)?;
    let canonical_target = target
        .requested_path()
        .canonicalize()
        .map_err(|_| verification("review target unavailable"))?;
    target.verify_current()?;
    let target_digest = sha256_digest(canonical_target.to_string_lossy().as_bytes());
    let scope_digest = sha256_digest(&encode(&(
        &target_digest,
        run_id,
        sha256_digest(&plan_bytes),
        &owner,
    ))?);
    let path = run_path(run_id, "POLICY-REVIEW.md")?;
    let previous = target.snapshot_bounded(&path, LIMIT)?;
    let mut book = if let Some(bytes) = previous.bytes() {
        let inner = bytes
            .strip_prefix(HEADER.as_bytes())
            .and_then(|bytes| bytes.strip_suffix(FOOTER.as_bytes()))
            .ok_or_else(|| verification("invalid review Markdown envelope"))?;
        let book: ReviewBook = serde_json::from_slice(inner)
            .map_err(|_| verification("invalid bounded review JSON"))?;
        book.validate()?;
        if book.scope_digest != scope_digest
            || book.host != owner.host
            || book.host_version_digest != sha256_digest(owner.host_version.as_bytes())
        {
            return Err(verification(
                "review belongs to another target, plan, or host binding",
            ));
        }
        book
    } else {
        ReviewBook {
            schema_version: 1,
            scope_digest,
            host: owner.host,
            host_version_digest: sha256_digest(owner.host_version.as_bytes()),
            operating_system: std::env::consts::OS.to_owned(),
            candidates: Vec::new(),
        }
    };
    let book_digest = sha256_digest(&book.bytes()?);
    let mut evaluation_snapshot = None;
    if matches!(verb, "preview" | "add") {
        let rule = required(&options, "--rule")?;
        if !RULES.contains(&rule) {
            return Err(verification("unregistered review rule"));
        }
        let relative = required(&options, "--evaluation")?;
        hive_core::validate_project_relative(Path::new(relative))
            .map_err(super::core_verification)?;
        let evaluation_path = run_path(run_id, relative)?;
        let bytes = target.read_required(&evaluation_path, LIMIT)?;
        let digest = sha256_digest(&bytes);
        let locator = format!(
            "{}#{digest}",
            super::portable_relative_path(&evaluation_path)
        );
        if !status.status().latest_evidence.contains(&locator)
            && !status
                .status()
                .criterion_evidence
                .values()
                .any(|values| values.contains(&locator))
        {
            return Err(verification(
                "evaluation is not registered in this run's reviewed evidence",
            ));
        }
        let evaluation: Evaluation = serde_json::from_slice(&bytes)
            .map_err(|_| verification("invalid bounded policy evaluation"))?;
        if evaluation.binding.operation_id != run_id
            || evaluation.binding.target_digest != target_digest
        {
            return Err(verification(
                "policy evaluation is bound to another operation or target",
            ));
        }
        let outcome = evaluate(&evaluation);
        if ![
            "hive.policy-denied",
            "hive.policy-incomplete",
            "hive.policy-unsupported",
            "hive.policy-checks-allow",
        ]
        .contains(&outcome.code.as_str())
        {
            return Err(verification(
                "policy evaluation has invalid structure or bindings",
            ));
        }
        book.observe(
            &evaluation,
            &digest,
            rule,
            class(required(&options, "--class")?)?,
        )?;
        evaluation_snapshot = Some((evaluation_path, bytes));
    } else if verb != "list" {
        if required(&options, "--confirm")? != book_digest {
            return Err(verification(
                "review confirmation does not match current candidates",
            ));
        }
        let id = required(&options, "--candidate")?;
        let candidate = book
            .candidates
            .iter_mut()
            .find(|candidate| candidate.id == id)
            .ok_or_else(|| verification("candidate not found"))?;
        if matches!(verb, "accept" | "revise") {
            let root = status_path.parent().expect("validated run status parent");
            for digest in candidate
                .supporting_evaluations
                .iter()
                .chain(&candidate.counter_evaluations)
            {
                let relative = status
                    .status()
                    .latest_evidence
                    .iter()
                    .chain(status.status().criterion_evidence.values().flatten())
                    .filter_map(|locator| locator.split_once('#'))
                    .find_map(|(path, expected)| {
                        (expected == digest && Path::new(path).starts_with(root))
                            .then_some(Path::new(path))
                    })
                    .ok_or_else(|| {
                        verification("candidate evidence is no longer registered in this run")
                    })?;
                if sha256_digest(&target.read_required(relative, LIMIT)?) != *digest {
                    return Err(verification("candidate evidence changed before review"));
                }
            }
        }
        candidate.state = match verb {
            "accept" => ReviewState::Accepted,
            "revise" => ReviewState::Revised,
            "reject" => ReviewState::Rejected,
            _ => ReviewState::Cancelled,
        };
        candidate.revised_class = if verb == "revise" {
            Some(class(required(&options, "--class")?)?)
        } else {
            None
        };
    }
    let desired = book.bytes()?;
    let preview_digest = sha256_digest(&encode(&(
        sha256_digest(&status_bytes),
        &book_digest,
        sha256_digest(&desired),
    ))?);
    let changed = if matches!(verb, "preview" | "list") {
        false
    } else {
        if verb == "add" && required(&options, "--confirm")? != preview_digest {
            return Err(verification(
                "candidate approval differs from current preview",
            ));
        }
        if target.read_required(&plan_path, LIMIT)? != plan_bytes
            || target.read_required(&status_path, LIMIT)? != status_bytes
        {
            return Err(verification("run changed during review"));
        }
        if let Some((path, bytes)) = evaluation_snapshot {
            if target.read_required(&path, LIMIT)? != bytes {
                return Err(verification("evaluation changed during review"));
            }
        }
        target.publish(&path, &previous, &desired)?
    };
    Ok(ActionResult {
        schema_version: 1,
        action: "ReviewPolicyCandidate",
        status: "success",
        exit_code: 0,
        code: "hive.policy-review",
        message: "bounded review only; cause unconfirmed and policy mutation not authorized"
            .to_owned(),
        changed_paths: if changed {
            vec![super::portable_relative_path(&path)]
        } else {
            Vec::new()
        },
        evidence: Vec::new(),
        next_action: None,
        data: Some(
            json!({"book":book,"book_digest":sha256_digest(&desired),"preview_digest":preview_digest,"target_digest":target_digest,
            "cause":"unconfirmed","authorizes_mutation":false,"actual_execution":"cli-review-only","host_hook_execution":"unverified"}),
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hive_core::policy::{Binding, Requirement, RuleResult};
    use std::fs;
    use tempfile::TempDir;

    fn fixture() -> (TempDir, Evaluation) {
        let temp = tempfile::tempdir().expect("temporary consumer");
        let target = temp.path().canonicalize().expect("canonical");
        fs::create_dir_all(target.join(".hive/runs/review-1/evidence")).expect("run directory");
        fs::write(
            target.join(".hive/runs/review-1/PLAN.md"),
            "# Plan\n\n- [ ] [build] Build succeeds\n",
        )
        .expect("plan");
        let binding = Binding {
            operation_id: "review-1".to_owned(),
            policy_digest: sha256_digest(b"policy"),
            target_digest: sha256_digest(target.to_string_lossy().as_bytes()),
        };
        let evaluation = Evaluation {
            schema_version: 1,
            binding: binding.clone(),
            requirements: vec![Requirement {
                rule_id: "exact-authority".to_owned(),
                mandatory: true,
            }],
            results: vec![RuleResult {
                rule_id: "exact-authority".to_owned(),
                binding,
                decision: Decision::Deny,
                code: "hive.denied".to_owned(),
                evidence_digest: None,
            }],
        };
        write_evaluation(temp.path(), &evaluation, &[]);
        (temp, evaluation)
    }

    fn write_evaluation(target: &Path, evaluation: &Evaluation, retained: &[String]) {
        let bytes = encode(evaluation).expect("evaluation");
        fs::write(
            target.join(".hive/runs/review-1/evidence/policy.json"),
            &bytes,
        )
        .expect("write evaluation");
        let mut locators = retained.to_vec();
        locators.push(format!(
            ".hive/runs/review-1/evidence/policy.json#{}",
            sha256_digest(&bytes)
        ));
        let status = json!({"schema_version":1,"run_id":"review-1","revision":1,"state":"executing",
            "required_criteria":["build"],"passed_criteria":[],"failed_criteria":[],"blocked_criteria":[],"active_roles":[],"next_action":"verify",
            "latest_evidence":locators,"blocker":null,"updated_at":"2026-09-19T00:00:00Z","host":"codex","host_version":"fixture",
            "surface":"cli","external_runtime":null,"resolved_owner":"host-native","resolution_evidence_digest":sha256_digest(b"capability"),
            "subagent_support":"supported","resume_note":null,"criterion_evidence":{}});
        fs::write(
            target.join(".hive/runs/review-1/STATUS.md"),
            format!(
                "---\n{}\n---\n# Status\n",
                serde_json::to_string(&status).expect("status")
            ),
        )
        .expect("write status");
    }

    fn call(target: &Path, verb: &str, extra: &[&str]) -> Result<ActionResult, AdapterError> {
        let mut args = vec![
            verb.to_owned(),
            "--target".to_owned(),
            target.to_str().expect("path").to_owned(),
            "--run".to_owned(),
            "review-1".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        args.extend(extra.iter().map(|value| (*value).to_owned()));
        execute(&args)
    }

    const PROPOSAL: &[&str] = &[
        "--evaluation",
        "evidence/policy.json",
        "--rule",
        "exact-authority",
        "--class",
        "non-compliance",
    ];

    fn add(target: &Path) -> serde_json::Value {
        let preview = call(target, "preview", PROPOSAL)
            .expect("preview")
            .data
            .expect("data");
        let mut extra = PROPOSAL.to_vec();
        extra.extend([
            "--confirm",
            preview["preview_digest"].as_str().expect("digest"),
        ]);
        call(target, "add", &extra)
            .expect("add")
            .data
            .expect("data")
    }

    #[test]
    fn preview_is_read_only_repeated_evidence_is_deduplicated_and_review_never_edits_policy() {
        let (temp, _) = fixture();
        let root = temp.path();
        fs::write(root.join("AGENTS.md"), b"external instructions\r\n").expect("foreign file");
        let status_before = fs::read(root.join(".hive/runs/review-1/STATUS.md")).expect("status");
        call(root, "preview", PROPOSAL).expect("preview");
        assert!(!root.join(".hive/runs/review-1/POLICY-REVIEW.md").exists());
        let first = add(root);
        let repeated = add(root);
        assert_eq!(first["book_digest"], repeated["book_digest"]);
        assert_eq!(first["book"], repeated["book"]);
        let id = first["book"]["candidates"][0]["id"].as_str().expect("id");
        let digest = first["book_digest"].as_str().expect("digest");
        let accepted = call(root, "accept", &["--candidate", id, "--confirm", digest])
            .expect("accept")
            .data
            .expect("data");
        assert_eq!(accepted["book"]["candidates"][0]["state"], "accepted");
        assert_eq!(accepted["authorizes_mutation"], false);
        let repeated = add(root);
        assert_eq!(repeated["book"]["candidates"][0]["state"], "accepted");
        assert_eq!(
            fs::read(root.join("AGENTS.md")).expect("foreign"),
            b"external instructions\r\n"
        );
        assert_eq!(
            fs::read(root.join(".hive/runs/review-1/STATUS.md")).expect("status"),
            status_before
        );
    }

    #[test]
    fn counterevidence_reopens_review_and_all_review_dispositions_remain_distinct() {
        let (temp, mut evaluation) = fixture();
        let first = add(temp.path());
        call(
            temp.path(),
            "accept",
            &[
                "--candidate",
                first["book"]["candidates"][0]["id"].as_str().expect("id"),
                "--confirm",
                first["book_digest"].as_str().expect("digest"),
            ],
        )
        .expect("accept before counterevidence");
        let old_bytes = encode(&evaluation).expect("old evaluation");
        fs::write(
            temp.path().join(".hive/runs/review-1/evidence/old.json"),
            &old_bytes,
        )
        .expect("retain");
        evaluation.results[0].decision = Decision::Allow;
        write_evaluation(
            temp.path(),
            &evaluation,
            &[format!(
                ".hive/runs/review-1/evidence/old.json#{}",
                sha256_digest(&old_bytes)
            )],
        );
        let mut current = add(temp.path());
        assert_eq!(current["book"]["candidates"][0]["state"], "pending");
        assert_eq!(
            current["book"]["candidates"]
                .as_array()
                .expect("candidates")
                .len(),
            1
        );
        assert_eq!(
            current["book"]["candidates"][0]["counter_evaluations"]
                .as_array()
                .expect("counter")
                .len(),
            1
        );
        let id = first["book"]["candidates"][0]["id"].as_str().expect("id");
        for verb in ["accept", "revise", "reject", "cancel"] {
            let mut extra = vec![
                "--candidate",
                id,
                "--confirm",
                current["book_digest"].as_str().expect("digest"),
            ];
            if verb == "revise" {
                extra.extend(["--class", "tool-environment"]);
            }
            current = call(temp.path(), verb, &extra)
                .expect("review")
                .data
                .expect("data");
            assert_eq!(current["authorizes_mutation"], false);
        }
        assert_eq!(current["book"]["candidates"][0]["state"], "cancelled");
    }

    #[test]
    fn stale_confirmation_changed_evidence_and_cross_target_binding_refuse_without_writes() {
        let (temp, mut evaluation) = fixture();
        let first = add(temp.path());
        let path = temp.path().join(".hive/runs/review-1/POLICY-REVIEW.md");
        let before = fs::read(&path).expect("book");
        let id = first["book"]["candidates"][0]["id"].as_str().expect("id");
        assert!(call(
            temp.path(),
            "accept",
            &["--candidate", id, "--confirm", "wrong"]
        )
        .is_err());
        fs::write(
            temp.path().join(".hive/runs/review-1/evidence/policy.json"),
            b"changed",
        )
        .expect("tamper");
        assert!(call(
            temp.path(),
            "accept",
            &[
                "--candidate",
                id,
                "--confirm",
                first["book_digest"].as_str().expect("digest")
            ]
        )
        .is_err());
        evaluation.binding.target_digest = sha256_digest(b"another target");
        evaluation.results[0].binding = evaluation.binding.clone();
        write_evaluation(temp.path(), &evaluation, &[]);
        assert!(call(temp.path(), "preview", PROPOSAL).is_err());
        assert_eq!(before, fs::read(path).expect("unchanged book"));
    }

    #[test]
    fn raw_extra_fields_unregistered_evidence_and_unregistered_rules_are_rejected() {
        let (temp, evaluation) = fixture();
        let mut value = serde_json::to_value(&evaluation).expect("value");
        value["transcript"] = json!("synthetic confidential text");
        fs::write(
            temp.path().join(".hive/runs/review-1/evidence/policy.json"),
            encode(&value).expect("bytes"),
        )
        .expect("raw input");
        assert!(call(temp.path(), "preview", PROPOSAL).is_err());
        assert!(serde_json::from_value::<Evaluation>(value).is_err());
        let mut invalid = PROPOSAL.to_vec();
        invalid[3] = "synthetic-secret-rule";
        assert!(call(temp.path(), "preview", &invalid).is_err());
        assert!(!temp
            .path()
            .join(".hive/runs/review-1/POLICY-REVIEW.md")
            .exists());
    }

    #[test]
    fn valid_book_matches_schema_and_source_or_allow_only_input_cannot_create_candidate() {
        let (temp, mut evaluation) = fixture();
        evaluation.results[0].decision = Decision::Allow;
        write_evaluation(temp.path(), &evaluation, &[]);
        assert!(call(temp.path(), "preview", PROPOSAL).is_err());
        evaluation.results[0].decision = Decision::Deny;
        write_evaluation(temp.path(), &evaluation, &[]);
        let output = add(temp.path());
        hive_core::validate_json_schema(
            include_str!("../../../../schemas/policy-review.schema.json"),
            &output["book"],
            "review book",
        )
        .expect("schema valid");
        fs::write(temp.path().join("hive-source.json"), "{}").expect("source marker");
        assert!(call(temp.path(), "list", &[]).is_err());
    }
}
