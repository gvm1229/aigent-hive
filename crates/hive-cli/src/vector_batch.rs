//! Explicit shared-only rebuild windows. No persistent process or cross-scope database.
use super::{
    changed_paths, clear_or_quarantine_staging, collection_corpus, collection_request,
    execution_budget, finish_build, invalid, io_error, optional, restore_staging, retain_build,
    scope_control, validate_build_result, value_digest, verify_runtime, worker, Corpus,
    ScopeControl, Selector, Target,
};
use hive_wiki::store::{RagStore, SharedSemanticPublication};
use hive_wiki::WikiError;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::time::{Duration, Instant};

#[cfg(test)]
mod tests {
    use super::{remaining_seconds, validate_window_inventory, WindowResult};
    use serde_json::json;
    use std::time::Duration;

    fn scopes() -> (
        tempfile::TempDir,
        Vec<super::Target>,
        Vec<(super::ScopeControl, String)>,
    ) {
        use crate::knowledge::vector::{
            contract_digest, InstalledRuntime, ScopeControl, Selector, Target,
        };
        use hive_wiki::rag::{RagVisibility, SemanticPartition};
        use hive_wiki::vector::VectorFiles;
        let work = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/work");
        std::fs::create_dir_all(&work).expect("work directory");
        let temporary = tempfile::tempdir_in(work).expect("root");
        let mut rows = Vec::new();
        for name in ["first", "second"] {
            let files = VectorFiles::open(temporary.path(), false).expect("files");
            let selector = Selector::Collection {
                partition: SemanticPartition {
                    collection_id: name.to_owned(),
                    visibility: RagVisibility::Shared,
                },
            };
            let scope_id = files.scope_id(&selector).expect("scope");
            let target = Target {
                files,
                selector: selector.clone(),
                scope_id,
                current_collection_id: None,
            };
            let control = ScopeControl {
                schema_version: 1,
                revision: 1,
                selector,
                enabled: true,
                consent_digest: "sha256:".to_owned() + &"a".repeat(64),
                runtime: InstalledRuntime {
                    id: "b".repeat(64),
                    python: temporary.path().join("must-not-run"),
                    identity: json!({}),
                    contract_digest: contract_digest().expect("contract"),
                    receipt_digest: "sha256:".to_owned() + &"c".repeat(64),
                    consent_digest: "sha256:".to_owned() + &"d".repeat(64),
                },
                checkpoint: None,
                active: None,
                previous: None,
                retired: Vec::new(),
            };
            let lease = target.files.writer(Some(&target.scope_id)).expect("lease");
            target
                .files
                .write_control(Some(&target.scope_id), None, &control)
                .expect("control");
            drop(lease);
            let digest = super::value_digest(&control).expect("digest");
            rows.push((target, (control, digest)));
        }
        rows.sort_by(|left, right| left.0.scope_id.cmp(&right.0.scope_id));
        let (targets, controls) = rows.into_iter().unzip();
        (temporary, targets, controls)
    }

    #[test]
    fn window_rechecks_control_after_scope_leases_before_runtime_execution() {
        let (_temporary, targets, controls) = scopes();
        let mut altered = controls[1].0.clone();
        altered.revision += 1;
        let lease = targets[1]
            .files
            .writer(Some(&targets[1].scope_id))
            .expect("lease");
        targets[1]
            .files
            .write_control(Some(&targets[1].scope_id), Some(&controls[1].1), &altered)
            .expect("external update");
        drop(lease);
        let mut phases = serde_json::Map::new();
        let error = super::run_window(
            &targets,
            &controls,
            false,
            1,
            std::time::Instant::now(),
            Duration::from_mins(1),
            &mut phases,
        )
        .expect_err("stale preflight");
        assert!(error.to_string().contains("changed after list validation"));
        assert_eq!(
            phases.keys().map(String::as_str).collect::<Vec<_>>(),
            ["preparation"]
        );
        for target in &targets {
            assert!(!target
                .files
                .database_path(&target.scope_id, hive_wiki::vector::DatabaseKind::Staging)
                .expect("staging path")
                .exists());
        }
    }

    #[test]
    fn window_list_never_substitutes_a_different_approved_runtime() {
        let (_temporary, targets, controls) = scopes();
        let mut altered = controls[1].0.clone();
        altered.runtime.id = "e".repeat(64);
        let lease = targets[1]
            .files
            .writer(Some(&targets[1].scope_id))
            .expect("lease");
        targets[1]
            .files
            .write_control(Some(&targets[1].scope_id), Some(&controls[1].1), &altered)
            .expect("other runtime");
        drop(lease);
        let error = super::rebuild_many(&targets, &[]).expect_err("runtime mismatch");
        assert!(error.to_string().contains("same approved runtime"));
    }

    #[test]
    fn remaining_budget_keeps_a_positive_fraction_of_the_last_second() {
        assert_eq!(remaining_seconds(Duration::ZERO), 0);
        assert_eq!(remaining_seconds(Duration::from_nanos(1)), 1);
        assert_eq!(remaining_seconds(Duration::from_nanos(999_999_999)), 1);
        assert_eq!(remaining_seconds(Duration::from_secs(1)), 1);
        assert_eq!(remaining_seconds(Duration::from_nanos(1_000_000_001)), 2);
    }

    #[test]
    fn phase_timing_preserves_success_and_error_without_recording_payloads() {
        let mut phases = serde_json::Map::new();
        let value = super::measure_phase(&mut phases, "first", || Ok(42)).expect("success");
        assert_eq!(value, 42);
        let result = super::measure_phase(&mut phases, "second", || {
            Err::<(), _>(hive_wiki::WikiError::Verification(
                "private-payload".to_owned(),
            ))
        });
        assert!(result
            .expect_err("same failure")
            .to_string()
            .contains("private-payload"));
        assert_eq!(phases.len(), 2);
        assert!(phases.values().all(|value| value
            .as_f64()
            .is_some_and(|seconds| seconds.is_finite() && seconds >= 0.0)));
        assert!(!serde_json::to_string(&phases)
            .expect("timings")
            .contains("private-payload"));
    }

    #[test]
    fn window_receipts_cover_each_started_and_unstarted_scope_exactly_once() {
        let valid =
            json!({"schema_version":1,"results":[{"index":0,"result":{}}],"not_started":[1,2]});
        let output: WindowResult = serde_json::from_value(valid.clone()).expect("receipt");
        assert_eq!(validate_window_inventory(&output, 3).expect("inventory"), 1);
        for invalid in [
            json!({"schema_version":1,"results":[{"index":1,"result":{}}],"not_started":[1,2]}),
            json!({"schema_version":1,"results":[{"index":0,"result":{}},{"index":0,"result":{}}],"not_started":[2]}),
            json!({"schema_version":1,"results":[{"index":0,"result":{}}],"not_started":[2]}),
            json!({"schema_version":2,"results":[{"index":0,"result":{}}],"not_started":[1,2]}),
        ] {
            let output = serde_json::from_value(invalid).expect("typed invalid receipt");
            assert!(validate_window_inventory(&output, 3).is_err());
        }
        assert!(validate_window_inventory(&output, 0).is_err());
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WindowResult {
    schema_version: u32,
    results: Vec<IndexedResult>,
    not_started: Vec<usize>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IndexedResult {
    index: usize,
    result: Value,
}

fn state(target: &Target, label: &str) -> Value {
    json!({"scope_id":target.scope_id,"selector":target.selector,"state":label})
}

fn remaining(started: Instant, budget: Duration) -> usize {
    remaining_seconds(budget.saturating_sub(started.elapsed()))
}

fn remaining_seconds(left: Duration) -> usize {
    usize::try_from(left.as_secs() + u64::from(left.subsec_nanos() != 0)).unwrap_or(0)
}

fn measure_phase<T>(
    phases: &mut Map<String, Value>,
    name: &str,
    operation: impl FnOnce() -> Result<T, WikiError>,
) -> Result<T, WikiError> {
    let started = Instant::now();
    let result = operation();
    phases.insert(name.to_owned(), json!(started.elapsed().as_secs_f64()));
    result
}

pub(super) fn rebuild_many(
    targets: &[Target],
    options: &[(&str, &str)],
) -> Result<Value, WikiError> {
    if targets.is_empty() || targets.len() > 100 {
        return Err(invalid("shared rebuild requires 1..100 explicit scopes"));
    }
    let (seconds, workers) = execution_budget(options)?;
    let fresh = match optional(options, "--rebuild-mode").unwrap_or("resume") {
        "resume" => false,
        "fresh" => true,
        _ => return Err(invalid("--rebuild-mode must be resume or fresh")),
    };
    let started = Instant::now();
    let budget = Duration::from_secs(u64::try_from(seconds).map_err(io_error)?);
    let mut controls = Vec::new();
    for target in targets {
        if !matches!(&target.selector, Selector::Collection { partition } if partition.visibility == hive_wiki::rag::RagVisibility::Shared)
            || target.files.root_path() != targets[0].files.root_path()
        {
            return Err(invalid(
                "vector windows require the same root and shared-only scopes",
            ));
        }
        let (control, digest) = scope_control(target)?;
        let control = control
            .filter(|value| value.enabled)
            .ok_or_else(|| invalid("every shared scope must be explicitly enabled"))?;
        controls.push((
            control,
            digest.ok_or_else(|| invalid("scope control digest is absent"))?,
        ));
    }
    if controls
        .iter()
        .any(|(control, _)| control.runtime != controls[0].0.runtime)
    {
        return Err(invalid("shared scopes require the same approved runtime"));
    }
    let preflight_seconds = started.elapsed().as_secs_f64();
    let mut windows = Vec::new();
    let mut results = Vec::new();
    let mut failure = None;
    let mut touched = false;
    for (window_number, window) in targets.chunks(16).enumerate() {
        let offset = window_number * 16;
        if remaining(started, budget) == 0 {
            results.extend(
                targets[offset..]
                    .iter()
                    .map(|target| state(target, "not-started")),
            );
            break;
        }
        touched = true;
        let window_started = Instant::now();
        let mut phases = Map::new();
        let result = run_window(
            window,
            &controls[offset..offset + window.len()],
            fresh,
            workers,
            started,
            budget,
            &mut phases,
        );
        windows.push(json!({"offset":offset,"elapsed_seconds":window_started.elapsed().as_secs_f64(),"phase_seconds":phases}));
        match result {
            Ok(rows) => results.extend(rows),
            Err(error) => {
                failure = Some(
                    json!({"code":error.code(),"message":error.to_string(),"window":window_number}),
                );
                results.extend(
                    window
                        .iter()
                        .map(|target| state(target, "failed-or-unpublished")),
                );
                results.extend(
                    targets[offset + window.len()..]
                        .iter()
                        .map(|target| state(target, "not-started")),
                );
                break;
            }
        }
    }
    let complete = results.iter().all(|row| row["state"] == "complete");
    Ok(
        json!({"complete":complete,"failed":failure.is_some(),"failure":failure,"scopes":results,
        "workers":workers,"max_seconds":seconds,"window_size":16,"elapsed_seconds":started.elapsed().as_secs_f64(),
        "timing":{"preflight_seconds":preflight_seconds,"windows":windows},
        "fts_unchanged":true,"changed_paths":if touched {changed_paths(&targets[0],true)} else {Vec::new()}}),
    )
}

fn run_window(
    targets: &[Target],
    expected: &[(ScopeControl, String)],
    fresh: bool,
    workers: usize,
    started: Instant,
    budget: Duration,
    phases: &mut Map<String, Value>,
) -> Result<Vec<Value>, WikiError> {
    // parse_shared_targets sorts IDs; retain that order across every writer lease.
    if targets
        .windows(2)
        .any(|pair| pair[0].scope_id >= pair[1].scope_id)
    {
        return Err(invalid("shared writer scopes must be unique and sorted"));
    }
    let runtime = &expected[0].0.runtime;
    let _leases = measure_phase(phases, "preparation", || {
        let leases = targets
            .iter()
            .map(|target| target.files.writer(Some(&target.scope_id)))
            .collect::<Result<Vec<_>, _>>()?;
        for (target, (_, expected_digest)) in targets.iter().zip(expected) {
            let (control, digest) = scope_control(target)?;
            if digest.as_deref() != Some(expected_digest.as_str())
                || !control.is_some_and(|control| control.enabled)
            {
                return Err(invalid("shared scope changed after list validation"));
            }
        }
        verify_runtime(&targets[0].files, runtime)?;
        Ok(leases)
    })?;
    let (store, corpora) = measure_phase(phases, "corpus", || {
        let requests = targets
            .iter()
            .map(collection_request)
            .collect::<Result<Vec<_>, _>>()?;
        let store = RagStore::open(targets[0].files.root_path())?;
        let corpora = store
            .shared_semantic_corpora_bounded(&requests, workers)?
            .into_iter()
            .zip(requests)
            .map(|(corpus, request)| collection_corpus(corpus, request))
            .collect::<Vec<_>>();
        Ok((store, corpora))
    })?;
    if remaining(started, budget) == 0 {
        return Ok(targets
            .iter()
            .map(|target| state(target, "not-started"))
            .collect());
    }
    let work = (|| {
        let databases = measure_phase(phases, "staging", || {
            let mut databases = Vec::new();
            for ((target, (before, _)), corpus) in targets.iter().zip(expected).zip(&corpora) {
                let digest = if fresh {
                    clear_or_quarantine_staging(target, before)?;
                    None
                } else {
                    restore_staging(target, before)?
                };
                let database = target.files.prepare_staging(&target.scope_id)?;
                databases.push(json!({"database":database,"chunks":corpus.chunks,"manifest_digest":corpus.manifest_digest,"expected_database_digest":digest}));
            }
            Ok(databases)
        })?;
        let seconds = remaining(started, budget);
        if seconds == 0 {
            return Ok(targets
                .iter()
                .map(|target| state(target, "prepared-not-started"))
                .collect());
        }
        let output = measure_phase(phases, "worker", || {
            let request = json!({"schema_version":1,"action":"build-many","runtime":targets[0].files.runtime_path(&runtime.id)?,
            "contract_digest":runtime.contract_digest,"workers":workers,"max_seconds":seconds,"databases":databases});
            if serde_json::to_vec(&request).map_err(io_error)?.len() > 256 * 1024 * 1024 {
                return Err(invalid("serialized vector window exceeds input limit"));
            }
            let value = worker(
                &targets[0].files,
                runtime,
                request,
                u64::try_from(seconds).map_err(io_error)? + 35,
            )?;
            serde_json::from_value::<WindowResult>(value).map_err(io_error)
        })?;
        measure_phase(phases, "publication", || {
            publish_window(targets, expected, &corpora, output, workers, &store)
        })
    })();
    if work.is_err() {
        // A failed worker response authenticates none of this window's working files.
        // Keep every file recoverable and attempt all quarantines while leases are held.
        let recovery_started = Instant::now();
        for target in targets {
            let _ = target.files.quarantine_staging(&target.scope_id);
        }
        phases.insert(
            "recovery".to_owned(),
            json!(recovery_started.elapsed().as_secs_f64()),
        );
    }
    work
}

fn publish_window(
    targets: &[Target],
    expected: &[(ScopeControl, String)],
    corpora: &[Corpus],
    output: WindowResult,
    workers: usize,
    store: &RagStore,
) -> Result<Vec<Value>, WikiError> {
    let count = validate_window_inventory(&output, targets.len())?;
    let verified = output
        .results
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            validate_build_result(&targets[index], corpora[index].chunks.len(), item.result)
        })
        .collect::<Result<Vec<_>, _>>()?;
    publish_verified(targets, expected, corpora, verified, workers, store, count)
}

fn validate_window_inventory(
    output: &WindowResult,
    target_count: usize,
) -> Result<usize, WikiError> {
    let count = output.results.len();
    if output.schema_version != 1
        || count > target_count
        || output
            .results
            .iter()
            .enumerate()
            .any(|(index, item)| item.index != index)
        || output.not_started != (count..target_count).collect::<Vec<_>>()
    {
        return Err(invalid(
            "vector worker returned an invalid window inventory",
        ));
    }
    Ok(count)
}

fn publish_verified(
    targets: &[Target],
    expected: &[(ScopeControl, String)],
    corpora: &[Corpus],
    verified: Vec<super::BuildResult>,
    workers: usize,
    store: &RagStore,
    count: usize,
) -> Result<Vec<Value>, WikiError> {
    let mut pending = Vec::new();
    for (index, result) in verified.into_iter().enumerate() {
        pending.push(retain_build(
            &targets[index],
            &expected[index].0,
            &corpora[index],
            result,
        )?);
    }
    if count > 0 {
        let bindings = corpora[..count]
            .iter()
            .map(|corpus| {
                Ok(SharedSemanticPublication {
                    request: corpus
                        .request
                        .as_ref()
                        .ok_or_else(|| invalid("shared request is absent"))?,
                    partition_digest: &corpus.manifest_digest,
                    authority_digest: corpus
                        .authority_digest
                        .as_deref()
                        .ok_or_else(|| invalid("shared authority is absent"))?,
                })
            })
            .collect::<Result<Vec<_>, WikiError>>()?;
        let after_digests = pending
            .iter()
            .map(|item| value_digest(&item.after))
            .collect::<Result<Vec<_>, _>>()?;
        store.with_shared_semantic_snapshots_bounded(
            &bindings,
            workers,
            |index| {
                targets[index].files.write_control(
                    Some(&targets[index].scope_id),
                    Some(&expected[index].1),
                    &pending[index].after,
                )
            },
            |index| {
                targets[index].files.write_control(
                    Some(&targets[index].scope_id),
                    Some(&after_digests[index]),
                    &expected[index].0,
                )
            },
        )?;
    }
    let mut rows = Vec::new();
    for (index, item) in pending.iter().enumerate() {
        let mut row = state(
            &targets[index],
            if item.result.complete {
                "complete"
            } else {
                "checkpoint"
            },
        );
        row["result"] = finish_build(&targets[index], item, workers);
        rows.push(row);
    }
    rows.extend(
        targets[count..]
            .iter()
            .map(|target| state(target, "prepared-not-started")),
    );
    Ok(rows)
}
