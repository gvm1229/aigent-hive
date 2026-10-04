//! Deterministic accounting of explicitly normalized host counters. No quota estimation.
use crate::native_workflow::GoalBudget;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Counters {
    pub input: u64,
    pub output: u64,
    pub cached_input: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub sequence: u64,
    pub counters: Counters,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Coverage {
    Own,
    IncludesChildren,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BudgetMetric {
    Input,
    InputAndOutput,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub source_digest: String,
    pub parent_digest: Option<String>,
    pub coverage: Coverage,
    pub cache_in_input: bool,
    pub baseline: Counters,
    pub baseline_sequence: u64,
    pub observations: Vec<Observation>,
    pub final_observed: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: u32,
    pub budget: u64,
    pub budget_metric: BudgetMetric,
    pub sources: Vec<Source>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
// Measurement completeness, arithmetic limit and runtime authority are independent facts.
#[allow(clippy::struct_excessive_bools)]
pub struct Report {
    pub schema_version: u32,
    pub input: u64,
    pub output: u64,
    pub total: u64,
    pub budget_metric: BudgetMetric,
    pub budget_used: u64,
    pub remaining: u64,
    pub overrun: u64,
    pub ignored_late_reports: u64,
    pub measurement_complete: bool,
    pub within_budget: bool,
    pub hard_cap_enforced: bool,
    pub authorizes_dispatch: bool,
}

fn normalized(c: Counters, inclusive: bool) -> Result<(u64, u64), &'static str> {
    let input = if inclusive {
        if c.cached_input > c.input {
            return Err("cached input exceeds inclusive input");
        }
        c.input
    } else {
        c.input
            .checked_add(c.cached_input)
            .ok_or("input counter overflow")?
    };
    Ok((input, c.output))
}

fn valid_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn delta(source: &Source) -> Result<(u64, u64, u64), &'static str> {
    let baseline = normalized(source.baseline, source.cache_in_input)?;
    let mut last = baseline;
    let mut sequence = Some(source.baseline_sequence);
    let mut ignored = 0;
    for event in &source.observations {
        let value = normalized(event.counters, source.cache_in_input)?;
        if let Some(previous) = sequence {
            if event.sequence < previous {
                if value.0 > last.0 || value.1 > last.1 {
                    return Err("late counter exceeds newer observation");
                }
                ignored += 1;
                continue;
            }
            if event.sequence == previous {
                if value != last {
                    return Err("duplicate sequence has different counters");
                }
                continue;
            }
        }
        if value.0 < last.0 || value.1 < last.1 {
            return Err("counter reset requires a new explicit measurement baseline");
        }
        sequence = Some(event.sequence);
        last = value;
    }
    Ok((last.0 - baseline.0, last.1 - baseline.1, ignored))
}

fn rollup(
    id: &str,
    sources: &BTreeMap<&str, &Source>,
    deltas: &BTreeMap<&str, (u64, u64)>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
) -> Result<(u64, u64), &'static str> {
    if visiting.len() >= 64 || !visiting.insert(id.to_owned()) {
        return Err("cyclic or excessive token ancestry");
    }
    let source = sources.get(id).ok_or("missing token source parent")?;
    let mut children = (0_u64, 0_u64);
    for child in sources
        .values()
        .filter(|s| s.parent_digest.as_deref() == Some(id))
    {
        let value = rollup(&child.source_digest, sources, deltas, visiting, visited)?;
        children.0 = children
            .0
            .checked_add(value.0)
            .ok_or("child input overflow")?;
        children.1 = children
            .1
            .checked_add(value.1)
            .ok_or("child output overflow")?;
    }
    visiting.remove(id);
    visited.insert(id.to_owned());
    let own = deltas[id];
    match source.coverage {
        Coverage::Own => Ok((
            own.0.checked_add(children.0).ok_or("input sum overflow")?,
            own.1.checked_add(children.1).ok_or("output sum overflow")?,
        )),
        Coverage::IncludesChildren if own.0 >= children.0 && own.1 >= children.1 => Ok(own),
        _ => Err("parent aggregate has not included its child observations"),
    }
}

/// Aggregate only counters whose inclusion relationship is known.
/// # Errors
/// Rejects unknown coverage, resets, cycles, absent parents and arithmetic overflow.
pub fn summarize(request: &Request) -> Result<Report, &'static str> {
    if request.schema_version != 1
        || request.budget == 0
        || request.sources.is_empty()
        || request.sources.len() > 1024
    {
        return Err("invalid token accounting request limits");
    }
    let mut sources = BTreeMap::new();
    let mut deltas = BTreeMap::new();
    let mut ignored = 0_u64;
    for source in &request.sources {
        if !valid_digest(&source.source_digest)
            || source
                .parent_digest
                .as_deref()
                .is_some_and(|d| !valid_digest(d))
            || source.coverage == Coverage::Unknown
            || source.observations.len() > 10_000
            || sources
                .insert(source.source_digest.as_str(), source)
                .is_some()
        {
            return Err("ambiguous token source identity or coverage");
        }
        let value = delta(source)?;
        ignored = ignored
            .checked_add(value.2)
            .ok_or("report count overflow")?;
        deltas.insert(source.source_digest.as_str(), (value.0, value.1));
    }
    for source in sources.values() {
        if source
            .parent_digest
            .as_deref()
            .is_some_and(|p| !sources.contains_key(p))
        {
            return Err("missing token source parent");
        }
    }
    let mut input = 0_u64;
    let mut output = 0_u64;
    let mut visited = BTreeSet::new();
    for source in sources.values().filter(|s| s.parent_digest.is_none()) {
        let value = rollup(
            &source.source_digest,
            &sources,
            &deltas,
            &mut BTreeSet::new(),
            &mut visited,
        )?;
        input = input.checked_add(value.0).ok_or("input sum overflow")?;
        output = output.checked_add(value.1).ok_or("output sum overflow")?;
    }
    if visited.len() != sources.len() {
        return Err("cyclic token source ancestry");
    }
    let total = input.checked_add(output).ok_or("total token overflow")?;
    let budget_used = match request.budget_metric {
        BudgetMetric::Input => input,
        BudgetMetric::InputAndOutput => total,
    };
    let mut budget = GoalBudget::new(request.budget);
    budget
        .allocate("measured-work", request.budget)
        .map_err(|_| "budget reservation failed")?;
    budget
        .refund("measured-work", request.budget.saturating_sub(budget_used))
        .map_err(|_| "budget settlement failed")?;
    Ok(Report {
        schema_version: 1,
        input,
        output,
        total,
        budget_metric: request.budget_metric,
        budget_used,
        remaining: budget.available,
        overrun: budget_used.saturating_sub(request.budget),
        ignored_late_reports: ignored,
        measurement_complete: request
            .sources
            .iter()
            .all(|s| s.final_observed && !s.observations.is_empty()),
        within_budget: budget_used <= request.budget,
        hard_cap_enforced: false,
        authorizes_dispatch: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source(id: &str, input: u64) -> Source {
        Source {
            source_digest: crate::sha256_digest(id.as_bytes()),
            parent_digest: None,
            coverage: Coverage::Own,
            cache_in_input: true,
            baseline: Counters {
                input: 100,
                output: 10,
                cached_input: 80,
            },
            baseline_sequence: 0,
            observations: vec![Observation {
                sequence: 1,
                counters: Counters {
                    input: 100 + input,
                    output: 20,
                    cached_input: 90,
                },
            }],
            final_observed: true,
        }
    }
    #[test]
    fn preserves_overrun_without_counting_cached_input_twice() {
        let request = Request {
            schema_version: 1,
            budget_metric: BudgetMetric::Input,
            budget: 1_000_000,
            sources: vec![source("root", 1_137_263)],
        };
        let report = summarize(&request).unwrap();
        assert_eq!(report.input, 1_137_263);
        assert_eq!(report.overrun, 137_263);
        assert_eq!(report.remaining, 0);
        assert!(!report.hard_cap_enforced);
    }
    #[test]
    fn parent_inclusive_and_exclusive_counters_are_distinct() {
        let mut parent = source("parent", 200);
        parent.coverage = Coverage::IncludesChildren;
        let mut child = source("child", 50);
        child.parent_digest = Some(parent.source_digest.clone());
        let mut request = Request {
            schema_version: 1,
            budget_metric: BudgetMetric::Input,
            budget: 1000,
            sources: vec![parent, child],
        };
        assert_eq!(summarize(&request).unwrap().input, 200);
        request.sources[0].coverage = Coverage::Own;
        assert_eq!(summarize(&request).unwrap().input, 250);
        request.sources[0].parent_digest = Some(request.sources[1].source_digest.clone());
        assert!(summarize(&request).is_err());
    }
    #[test]
    fn ignores_old_reports_but_refuses_resets_and_conflicting_duplicates() {
        let mut s = source("root", 200);
        s.observations[0].sequence = 2;
        s.observations.push(Observation {
            sequence: 1,
            counters: Counters {
                input: 150,
                output: 15,
                cached_input: 90,
            },
        });
        let mut r = Request {
            schema_version: 1,
            budget_metric: BudgetMetric::Input,
            budget: 1000,
            sources: vec![s],
        };
        assert_eq!(summarize(&r).unwrap().ignored_late_reports, 1);
        r.sources[0].observations[1].sequence = 3;
        assert!(summarize(&r).is_err());
        r.sources[0].observations[1].sequence = 2;
        assert!(summarize(&r).is_err());
    }
    #[test]
    fn missing_parent_unknown_coverage_and_separate_cache_are_explicit() {
        let mut s = source("root", 20);
        s.cache_in_input = false;
        let mut r = Request {
            schema_version: 1,
            budget_metric: BudgetMetric::Input,
            budget: 1000,
            sources: vec![s],
        };
        assert_eq!(summarize(&r).unwrap().input, 30);
        r.sources[0].coverage = Coverage::Unknown;
        assert!(summarize(&r).is_err());
        r.sources[0].coverage = Coverage::Own;
        r.sources[0].parent_digest = Some(crate::sha256_digest(b"absent"));
        assert!(summarize(&r).is_err());
    }

    #[test]
    fn parent_rollup_must_cover_all_children_not_only_each_one() {
        let mut parent = source("parent", 100);
        parent.coverage = Coverage::IncludesChildren;
        let mut a = source("a", 80);
        a.parent_digest = Some(parent.source_digest.clone());
        let mut b = source("b", 80);
        b.parent_digest = Some(parent.source_digest.clone());
        let r = Request {
            schema_version: 1,
            budget: 1000,
            budget_metric: BudgetMetric::Input,
            sources: vec![parent, a, b],
        };
        assert!(summarize(&r).is_err());
    }

    #[test]
    fn reports_before_baseline_are_not_charged() {
        let mut s = source("root", 20);
        s.baseline_sequence = 10;
        s.observations[0].sequence = 11;
        s.observations.insert(
            0,
            Observation {
                sequence: 9,
                counters: Counters {
                    input: 90,
                    output: 9,
                    cached_input: 70,
                },
            },
        );
        let r = Request {
            schema_version: 1,
            budget: 1000,
            budget_metric: BudgetMetric::InputAndOutput,
            sources: vec![s],
        };
        let report = summarize(&r).unwrap();
        assert_eq!(report.input, 20);
        assert_eq!(report.total, 30);
        assert_eq!(report.remaining, 970);
        assert_eq!(report.ignored_late_reports, 1);
    }
}
