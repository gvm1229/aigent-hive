//! Pure policy-result aggregation. Evaluations are evidence, never mutation authority.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Closed outcomes shared by domain validators and host adapters.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Decision {
    /// Every mandatory check supplied an allowing result.
    Allow,
    /// A check explicitly refused the operation.
    Deny,
    /// A mandatory capability is not supported.
    Unsupported,
    /// Required evidence is absent, inconsistent, or invalid.
    Error,
}

/// Exact operation identity determined by the calling domain, not by hook prose.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    /// Bounded, non-path operation identifier.
    pub operation_id: String,
    /// Content digest of the active policy.
    pub policy_digest: String,
    /// Digest of the authenticated target identity.
    pub target_digest: String,
}

/// One policy rule selected before any checker executes.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    /// Stable rule identifier.
    pub rule_id: String,
    /// Missing or unsuccessful results must withhold the operation.
    pub mandatory: bool,
}

/// A bounded validator result. Producers remain responsible for real evidence.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleResult {
    /// Rule that produced this result.
    pub rule_id: String,
    /// Operation, policy, and target that were checked.
    pub binding: Binding,
    /// Validator outcome.
    pub decision: Decision,
    /// A stable diagnostic code, never arbitrary commands or transcript text.
    pub code: String,
    /// Optional digest of the bounded evidence inspected by the validator.
    pub evidence_digest: Option<String>,
}

/// Side-effect-free evaluation input.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Evaluation {
    /// Input contract generation.
    pub schema_version: u32,
    /// Current exact binding.
    pub binding: Binding,
    /// Complete expected rule set, including non-blocking diagnostics.
    pub requirements: Vec<Requirement>,
    /// Results may arrive in any order.
    pub results: Vec<RuleResult>,
}

/// Aggregate outcome with explicit coverage and no implied execution authority.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct Outcome {
    /// Contract generation.
    pub schema_version: u32,
    /// Overall diagnostic decision.
    pub decision: Decision,
    /// Stable reason for the aggregate decision.
    pub code: String,
    /// Successfully matched result IDs, sorted.
    pub observed_rules: Vec<String>,
    /// Mandatory rules with no result, sorted.
    pub missing_rules: Vec<String>,
    /// Non-blocking diagnostics that were not successful, sorted.
    pub advisory_rules: Vec<String>,
    /// Always false: callers must revalidate at their actual mutation boundary.
    pub authorizes_mutation: bool,
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
}

/// Check the canonical SHA-256 spelling without interpreting it as authority.
#[must_use]
pub fn valid_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn invalid(code: &str) -> Outcome {
    Outcome {
        schema_version: 1,
        decision: Decision::Error,
        code: code.to_owned(),
        observed_rules: Vec::new(),
        missing_rules: Vec::new(),
        advisory_rules: Vec::new(),
        authorizes_mutation: false,
    }
}

/// Join checker results by identity, never by arrival order or array position.
///
/// Input validity and coverage are checked before any allowing result is emitted.
/// An evaluation does not establish that an external checker is trustworthy.
#[must_use]
pub fn evaluate(request: &Evaluation) -> Outcome {
    if request.schema_version != 1 {
        return invalid("hive.policy-unsupported-schema");
    }
    if !valid_id(&request.binding.operation_id)
        || !valid_digest(&request.binding.policy_digest)
        || !valid_digest(&request.binding.target_digest)
    {
        return invalid("hive.policy-invalid-binding");
    }
    if request.requirements.is_empty()
        || request.requirements.len() > 128
        || request.results.len() > 128
    {
        return invalid("hive.policy-invalid-rule-count");
    }
    let mut requirements = BTreeMap::new();
    for requirement in &request.requirements {
        if !valid_id(&requirement.rule_id)
            || requirements
                .insert(requirement.rule_id.as_str(), requirement.mandatory)
                .is_some()
        {
            return invalid("hive.policy-invalid-requirements");
        }
    }
    if !requirements.values().any(|mandatory| *mandatory) {
        return invalid("hive.policy-no-mandatory-rule");
    }
    let mut results = BTreeMap::new();
    for result in &request.results {
        if !requirements.contains_key(result.rule_id.as_str()) {
            return invalid("hive.policy-unknown-result");
        }
        if results.insert(result.rule_id.as_str(), result).is_some() {
            return invalid("hive.policy-duplicate-result");
        }
        if result.binding != request.binding {
            return invalid("hive.policy-result-binding-mismatch");
        }
        if !valid_id(&result.code)
            || result
                .evidence_digest
                .as_deref()
                .is_some_and(|digest| !valid_digest(digest))
        {
            return invalid("hive.policy-invalid-result");
        }
    }
    let mut missing = Vec::new();
    let mut advisory = Vec::new();
    let mut denied = false;
    let mut failed = false;
    let mut unsupported = false;
    for (rule, mandatory) in requirements {
        match results.get(rule) {
            Some(result) if result.decision == Decision::Deny => denied = true,
            Some(result) if mandatory => match result.decision {
                Decision::Error => failed = true,
                Decision::Unsupported => unsupported = true,
                Decision::Allow | Decision::Deny => {}
            },
            Some(result) if result.decision != Decision::Allow => advisory.push(rule.to_owned()),
            None if mandatory => missing.push(rule.to_owned()),
            None => advisory.push(rule.to_owned()),
            Some(_) => {}
        }
    }
    let (decision, code) = if denied {
        (Decision::Deny, "hive.policy-denied")
    } else if failed || !missing.is_empty() {
        (Decision::Error, "hive.policy-incomplete")
    } else if unsupported {
        (Decision::Unsupported, "hive.policy-unsupported")
    } else {
        (Decision::Allow, "hive.policy-checks-allow")
    };
    Outcome {
        schema_version: 1,
        decision,
        code: code.to_owned(),
        observed_rules: results.keys().map(|key| (*key).to_owned()).collect(),
        missing_rules: missing,
        advisory_rules: advisory,
        authorizes_mutation: false,
    }
}

/// Proposed failure cause; attribution always requires review.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FailureClass {
    /// No existing rule addressed the failure.
    MissingRule,
    /// An existing rule was not followed.
    NonCompliance,
    /// Following a rule produced harm.
    HarmfulRule,
    /// A tool or environment failed.
    ToolEnvironment,
    /// The available evidence does not establish the cause.
    InsufficientEvidence,
}

/// A review suggestion, not a policy edit or a verified causal claim.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct FailureProposal {
    /// Proposed failure class.
    pub proposed_class: FailureClass,
    /// Stable next-review action.
    pub next_action: String,
    /// Sorted, unique bounded evidence digests.
    pub evidence: Vec<String>,
    /// Always true; no model claim automatically updates policy.
    pub review_required: bool,
    /// Always unconfirmed: bounded evidence does not establish causal attribution.
    pub certainty: &'static str,
    /// Owning review area, not an automatically assigned or executed task.
    pub review_area: &'static str,
}

/// Prepare a bounded causal-review candidate without collecting raw input.
#[must_use]
pub fn propose_failure(class: FailureClass, evidence: &[String]) -> FailureProposal {
    let valid = !evidence.is_empty()
        && evidence.len() <= 32
        && evidence.iter().all(|item| valid_digest(item));
    let proposed_class = if valid {
        class
    } else {
        FailureClass::InsufficientEvidence
    };
    let next_action = match proposed_class {
        FailureClass::MissingRule => "review-new-rule",
        FailureClass::NonCompliance => "review-enforcement-boundary",
        FailureClass::HarmfulRule => "review-rule-change-or-removal",
        FailureClass::ToolEnvironment => "repair-tool-or-environment",
        FailureClass::InsufficientEvidence => "collect-bounded-evidence",
    };
    FailureProposal {
        proposed_class,
        next_action: next_action.to_owned(),
        evidence: if valid {
            evidence
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect()
        } else {
            Vec::new()
        },
        review_required: true,
        certainty: "unconfirmed",
        review_area: match proposed_class {
            FailureClass::MissingRule | FailureClass::HarmfulRule => "policy-review",
            FailureClass::NonCompliance => "enforcement-boundary",
            FailureClass::ToolEnvironment => "tool-environment",
            FailureClass::InsufficientEvidence => "evidence-review",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> Evaluation {
        let binding = Binding {
            operation_id: "op-1".to_owned(),
            policy_digest: format!("sha256:{}", "a".repeat(64)),
            target_digest: format!("sha256:{}", "b".repeat(64)),
        };
        Evaluation {
            schema_version: 1,
            binding: binding.clone(),
            requirements: vec![
                Requirement {
                    rule_id: "path".to_owned(),
                    mandatory: true,
                },
                Requirement {
                    rule_id: "authority".to_owned(),
                    mandatory: true,
                },
            ],
            results: ["path", "authority"]
                .into_iter()
                .map(|id| RuleResult {
                    rule_id: id.to_owned(),
                    binding: binding.clone(),
                    decision: Decision::Allow,
                    code: "checked".to_owned(),
                    evidence_digest: None,
                })
                .collect(),
        }
    }

    #[test]
    fn result_order_is_irrelevant_and_allow_never_grants_authority() {
        let mut input = request();
        let first = evaluate(&input);
        input.results.reverse();
        assert_eq!(first, evaluate(&input));
        assert_eq!(first.decision, Decision::Allow);
        assert!(!first.authorizes_mutation);
    }

    #[test]
    fn schema_and_rust_contract_agree_and_unknown_versions_fail_closed() {
        let mut input = request();
        crate::validate_json_schema(
            include_str!("../../../schemas/policy-evaluation.schema.json"),
            &serde_json::to_value(&input).expect("serialize"),
            "policy evaluation",
        )
        .expect("valid schema");
        input.schema_version = 2;
        assert_eq!(evaluate(&input).code, "hive.policy-unsupported-schema");
    }

    #[test]
    fn missing_duplicate_and_unknown_results_never_allow() {
        let mut input = request();
        input.results.pop();
        assert_eq!(evaluate(&input).missing_rules, ["authority"]);
        input.results.push(input.results[0].clone());
        assert_eq!(evaluate(&input).code, "hive.policy-duplicate-result");
        input.results[1].rule_id = "injected".to_owned();
        assert_eq!(evaluate(&input).code, "hive.policy-unknown-result");
    }

    #[test]
    fn stale_operation_policy_and_target_are_rejected() {
        for field in 0..3 {
            let mut input = request();
            match field {
                0 => input.results[0].binding.operation_id = "other-op".to_owned(),
                1 => input.results[0].binding.policy_digest = format!("sha256:{}", "c".repeat(64)),
                _ => input.results[0].binding.target_digest = format!("sha256:{}", "d".repeat(64)),
            }
            assert_eq!(evaluate(&input).code, "hive.policy-result-binding-mismatch");
        }
    }

    #[test]
    fn denial_wins_and_mandatory_failures_remain_distinct() {
        let mut input = request();
        input.results[0].decision = Decision::Unsupported;
        assert_eq!(evaluate(&input).decision, Decision::Unsupported);
        input.results[1].decision = Decision::Error;
        assert_eq!(evaluate(&input).decision, Decision::Error);
        input.results[0].decision = Decision::Deny;
        assert_eq!(evaluate(&input).decision, Decision::Deny);
    }

    #[test]
    fn optional_diagnostics_do_not_disable_mandatory_protection() {
        let mut input = request();
        input.requirements[1].mandatory = false;
        input.results[1].decision = Decision::Error;
        assert_eq!(evaluate(&input).decision, Decision::Allow);
        assert_eq!(evaluate(&input).advisory_rules, ["authority"]);
        input.results[1].decision = Decision::Deny;
        assert_eq!(evaluate(&input).decision, Decision::Deny);
    }

    #[test]
    fn empty_duplicate_or_malformed_requirements_are_rejected() {
        let mut input = request();
        input.requirements.push(input.requirements[0].clone());
        assert_eq!(evaluate(&input).decision, Decision::Error);
        input.requirements.clear();
        assert_eq!(evaluate(&input).decision, Decision::Error);
        let mut input = request();
        input.binding.operation_id = "private/path".to_owned();
        assert_eq!(evaluate(&input).decision, Decision::Error);
    }

    #[test]
    fn causal_attribution_is_a_review_proposal_and_never_an_automatic_edit() {
        let evidence = vec![format!("sha256:{}", "e".repeat(64))];
        for class in [
            FailureClass::MissingRule,
            FailureClass::NonCompliance,
            FailureClass::HarmfulRule,
            FailureClass::ToolEnvironment,
        ] {
            let proposal = propose_failure(class, &evidence);
            assert_eq!(proposal.proposed_class, class);
            assert!(proposal.review_required);
            assert_eq!(
                propose_failure(class, &[]).proposed_class,
                FailureClass::InsufficientEvidence
            );
        }
        assert_eq!(
            propose_failure(FailureClass::MissingRule, &["raw transcript".to_owned()])
                .evidence
                .len(),
            0
        );
    }
}
