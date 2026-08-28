//! Separate one-time authority for confidential index maintenance; retrieval tokens cannot grant it.
use super::{invalid, optional, required, scope_control, value_digest, Selector, Target};
use crate::knowledge;
use hive_wiki::rag::{RagVisibility, RetrievalRequest, RetrievalScope};
use hive_wiki::store::RagStore;
use hive_wiki::WikiError;
use serde_json::{json, Value};
use std::path::Path;

pub(super) fn confidential(target: &Target) -> bool {
    matches!(&target.selector, Selector::Collection {partition} if partition.visibility==RagVisibility::Confidential)
}

pub(super) fn validate_fields(target: &Target, options: &[(&str, &str)]) -> Result<(), WikiError> {
    let fields = [
        "--authorization-id",
        "--authorization-token",
        "--capabilities",
        "--usage",
    ];
    let count = fields
        .iter()
        .filter(|name| optional(options, name).is_some())
        .count();
    if (confidential(target) && count != 4) || (!confidential(target) && count != 0) {
        return Err(invalid("confidential maintenance needs its own ID, token, capability and usage snapshots; ordinary scopes reject these fields"));
    }
    Ok(())
}

fn collection(target: &Target) -> Result<&str, WikiError> {
    match &target.selector {
        Selector::Collection { partition }
            if partition.visibility == RagVisibility::Confidential =>
        {
            Ok(&partition.collection_id)
        }
        _ => Err(invalid(
            "build authorization is only for a confidential consumer collection",
        )),
    }
}

fn binding(
    target: &Target,
    options: &[(&str, &str)],
    operation: &str,
    manifest: &str,
) -> Result<(Value, String), WikiError> {
    if !["rebuild", "rollback"].contains(&operation) {
        return Err(invalid(
            "build authorization operation must be rebuild or rollback",
        ));
    }
    let collection = collection(target)?;
    let store = RagStore::open(target.files.root_path())?;
    let (registry, registry_digest) = store.load_registry_snapshot()?;
    let (canonical, current) = knowledge::derive_current_collection_authority(
        &registry,
        Path::new(required(options, "--target")?),
    )?;
    if target.current_collection_id.as_deref() != Some(&current) {
        return Err(invalid(
            "current collection changed before vector authorization",
        ));
    }
    let (control, control_digest) = scope_control(target)?;
    let control = control
        .filter(|control| control.enabled)
        .ok_or_else(|| invalid("enable the confidential scope before authorizing its build"))?;
    let (seconds, workers) = super::index::execution_budget(options)?;
    let mode = optional(options, "--rebuild-mode").unwrap_or("resume");
    if !["resume", "fresh"].contains(&mode) {
        return Err(invalid("invalid rebuild mode"));
    }
    if operation == "rollback"
        && ["--max-seconds", "--workers", "--rebuild-mode"]
            .iter()
            .any(|name| optional(options, name).is_some())
    {
        return Err(invalid(
            "rollback authorization rejects rebuild budget options",
        ));
    }
    Ok((
        json!({"schema_version":1,"action":"confidential-vector-maintenance","operation":operation,
        "collection_id":collection,"current_collection_id":current,"current_target_digest":knowledge::canonical_target_digest(&canonical),
        "registry_digest":registry_digest,"manifest_digest":manifest,"scope_id":target.scope_id,"scope_control_digest":control_digest,
        "runtime_contract":control.runtime.contract_digest,"runtime_receipt":control.runtime.receipt_digest,
        "max_seconds":if operation=="rebuild" {seconds} else {0},"workers":if operation=="rebuild" {workers} else {0},
        "rebuild_mode":if operation=="rebuild" {mode} else {"none"}}),
        current,
    ))
}

fn authorization_binding(
    options: &[(&str, &str)],
    action_digest: &str,
    expires: u64,
    nonce: &str,
) -> Result<String, WikiError> {
    let capabilities = knowledge::read_snapshot_digest(
        Path::new(required(options, "--capabilities")?),
        "capability snapshot",
    )?;
    let usage = knowledge::read_snapshot_digest(
        Path::new(required(options, "--usage")?),
        "usage snapshot",
    )?;
    value_digest(&knowledge::AuthorizationBinding {
        schema_version: 1,
        canonical_action_digest: action_digest,
        capability_snapshot_digest: Some(&capabilities),
        usage_snapshot_digest: Some(&usage),
        expires_at_unix_seconds: expires,
        nonce,
    })
}

pub(super) fn issue(target: &Target, options: &[(&str, &str)]) -> Result<Value, WikiError> {
    let collection = collection(target)?;
    let operation = optional(options, "--operation").unwrap_or("rebuild");
    let nonce = required(options, "--nonce")?;
    knowledge::validate_authorization_nonce(nonce)?;
    let now = knowledge::unix_now()?;
    let expires = required(options, "--expires-at")?
        .parse::<u64>()
        .map_err(|_| invalid("authorization expiry must be Unix seconds"))?;
    if expires <= now || expires - now > knowledge::MAX_AUTHORIZATION_TTL_SECONDS {
        return Err(invalid("authorization must start within sixty seconds"));
    }
    let store = RagStore::open(target.files.root_path())?;
    let request = RetrievalRequest {
        scope: RetrievalScope::Collection(collection.to_owned()),
        current_collection_id: target.current_collection_id.clone(),
        query: "vector authorization metadata".to_owned(),
        query_expansions: Vec::new(),
        top_k: 1,
        byte_budget: 1,
        confidential_collection_id: None,
    };
    // Metadata only; no confidential text is exported to issue a grant.
    let manifest = store.retrieve(&request)?.manifest_digest;
    let (action, current) = binding(target, options, operation, &manifest)?;
    let digest = value_digest(&action)?;
    let bound = authorization_binding(options, &digest, expires, nonce)?;
    let id = knowledge::generate_authorization_secret()?;
    let token = knowledge::generate_authorization_secret()?;
    let record = knowledge::ConfidentialAuthorizationRecord {
        schema_version: 1,
        authorization_id: id.clone(),
        token_digest: hive_core::sha256_digest(token.as_bytes()),
        canonical_action_digest: digest.clone(),
        authorization_binding_digest: bound,
        action: knowledge::AuthorizationAction::ConfidentialVectorBuild {
            collection_id: collection.to_owned(),
            current_collection_id: current,
            request_binding_digest: digest,
        },
        issued_at_unix_seconds: now,
        expires_at_unix_seconds: expires,
        nonce: nonce.to_owned(),
    };
    let (relative, record_digest) =
        knowledge::issue_authorization(target.files.root_path(), &store, &record)?;
    Ok(
        json!({"authorization_id":id,"authorization_token":token,"one_time":true,"operation":operation,
        "expires_at_unix_seconds":expires,"manifest_digest":manifest,"record_digest":record_digest,
        "changed_paths":[relative.to_string_lossy()]}),
    )
}

/// Called only inside `RagStore::authorized_semantic_corpus`, under its canonical lock.
pub(super) fn consume_locked(
    target: &Target,
    options: &[(&str, &str)],
    operation: &str,
    manifest: &str,
) -> Result<(), WikiError> {
    validate_fields(target, options)?;
    let id = required(options, "--authorization-id")?;
    let token = required(options, "--authorization-token")?;
    knowledge::validate_authorization_credentials(id, token)?;
    let (canonical, root) = knowledge::authorization_root(target.files.root_path(), false)?;
    knowledge::reject_consumed_authorization(&root, id)?;
    let relative = knowledge::authorization_record_relative(id, false)?;
    let bytes = knowledge::read_authorization_record(&canonical, &root, &relative)?;
    let record = knowledge::parse_authorization_record(&bytes)?;
    knowledge::validate_authorization_record_common(&record, id, token)?;
    let knowledge::AuthorizationAction::ConfidentialVectorBuild {
        collection_id,
        current_collection_id,
        request_binding_digest,
    } = &record.action
    else {
        return Err(invalid(
            "retrieval or mapping authorization cannot authorize a confidential vector build",
        ));
    };
    let (action, current) = binding(target, options, operation, manifest)?;
    let digest = value_digest(&action)?;
    let bound = authorization_binding(
        options,
        &digest,
        record.expires_at_unix_seconds,
        &record.nonce,
    )?;
    if collection_id != collection(target)?
        || current_collection_id != &current
        || request_binding_digest != &digest
        || record.canonical_action_digest != digest
        || record.authorization_binding_digest != bound
    {
        return Err(invalid(
            "confidential vector authorization no longer matches this exact action",
        ));
    }
    knowledge::consume_authorization_record(&canonical, &root, &relative, &bytes, &record)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::vector::{InstalledRuntime, ScopeControl};
    use hive_core::sha256_digest;
    use hive_wiki::rag::{plan_remember, RememberRequest, SemanticPartition};
    use hive_wiki::vector::VectorFiles;
    use std::{fs, path::PathBuf};

    type Arguments = Vec<(String, String)>;

    fn borrowed(values: &Arguments) -> Vec<(&str, &str)> {
        values
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect()
    }

    fn fixture() -> (tempfile::TempDir, Target, RagStore, Arguments) {
        let work = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/work");
        let temporary = tempfile::tempdir_in(work).expect("temporary root");
        let root = temporary.path().canonicalize().expect("canonical root");
        let store = RagStore::open(&root).expect("store");
        store.ensure_registry().expect("registry");
        fs::write(root.join(".hive/config/user-setup.yml"),serde_json::to_vec(&json!({
            "schema_version":1,"interface_language":"en","wiki":{"enabled":true,"language":"both"},
            "profile":{"id":"web-developer"},"persona":{"id":"balanced"},"selected_hosts":["codex"],
            "skills":{"mode":"individual","selected":["setup-hive"]},
            "usage_guard":{"enabled":false,"stop_remaining_percent":20,"codexbar_fallback_enabled":false}
        })).expect("setup JSON")).expect("setup");
        let id = format!(
            "claim-{}",
            &sha256_digest(b"synthetic confidential claim")[7..]
        );
        let request:RememberRequest=serde_json::from_value(json!({"collection_id":"user-root","claim_key":"synthetic-confidential",
            "claim_id":id,"locator":format!(".hive/knowledge/Claims/user-root/{id}.md"),"kind":"decision","status":"user-stated",
            "visibility":"confidential","normalized_fact":"Keep synthetic confidential retrieval evidence local.",
            "provenance":{"source_kind":"user-statement","summary":"Synthetic current action","locator":"request:synthetic","digest":sha256_digest(b"synthetic")},
            "sources":["request:synthetic"],"supersedes":[]})).expect("claim request");
        store
            .apply_remember_plan(&plan_remember(&[], &request, 2).expect("claim plan"))
            .expect("claim");
        let files = VectorFiles::open(&root, false).expect("files");
        let selector = Selector::Collection {
            partition: SemanticPartition {
                collection_id: "user-root".to_owned(),
                visibility: RagVisibility::Confidential,
            },
        };
        let scope_id = files.scope_id(&selector).expect("scope");
        let control = ScopeControl {
            schema_version: 1,
            revision: 1,
            selector: selector.clone(),
            enabled: true,
            consent_digest: sha256_digest(b"consent"),
            runtime: InstalledRuntime {
                id: "a".repeat(64),
                python: PathBuf::new(),
                identity: json!({}),
                contract_digest: sha256_digest(b"contract"),
                receipt_digest: sha256_digest(b"receipt"),
                consent_digest: sha256_digest(b"consent"),
            },
            checkpoint: None,
            active: None,
            previous: None,
            retired: Vec::new(),
        };
        files
            .write_control(Some(&scope_id), None, &control)
            .expect("synthetic approved scope");
        let target = Target {
            files,
            selector,
            scope_id,
            current_collection_id: Some("user-root".to_owned()),
        };
        let capabilities = root.join("capabilities.json");
        let usage = root.join("usage.json");
        fs::write(&capabilities, b"{\"schema_version\":1,\"synthetic\":true}")
            .expect("capabilities");
        fs::write(&usage, b"{\"schema_version\":1,\"synthetic\":true}").expect("usage");
        let args = vec![
            ("--target".to_owned(), root.to_string_lossy().into_owned()),
            (
                "--capabilities".to_owned(),
                capabilities.to_string_lossy().into_owned(),
            ),
            ("--usage".to_owned(), usage.to_string_lossy().into_owned()),
            (
                "--expires-at".to_owned(),
                (knowledge::unix_now().expect("time") + 60).to_string(),
            ),
            ("--nonce".to_owned(), "synthetic-approval".to_owned()),
        ];
        (temporary, target, store, args)
    }

    fn with_token(args: &Arguments, grant: &Value) -> Arguments {
        let mut values = args.clone();
        values.push((
            "--authorization-id".to_owned(),
            grant["authorization_id"].as_str().expect("ID").to_owned(),
        ));
        values.push((
            "--authorization-token".to_owned(),
            grant["authorization_token"]
                .as_str()
                .expect("token")
                .to_owned(),
        ));
        values
    }

    fn export(
        target: &Target,
        store: &RagStore,
        args: &Arguments,
        operation: &str,
    ) -> Result<hive_wiki::rag::SemanticCorpus, WikiError> {
        let request = RetrievalRequest {
            scope: RetrievalScope::Collection("user-root".to_owned()),
            current_collection_id: Some("user-root".to_owned()),
            query: "vector build".to_owned(),
            query_expansions: Vec::new(),
            top_k: 100,
            byte_budget: 4096,
            confidential_collection_id: Some("user-root".to_owned()),
        };
        store.authorized_semantic_corpus(&request, RagVisibility::Confidential, |manifest| {
            consume_locked(target, &borrowed(args), operation, manifest)
        })
    }

    #[test]
    fn cpu_capacity_drift_cannot_change_the_authorized_execution_budget() {
        let (_root, target, store, args) = fixture();
        super::super::TEST_DEFAULT_WORKERS.with(|value| value.set(Some(4)));
        let grant = issue(&target, &borrowed(&args)).expect("four workers approved");
        let consuming = with_token(&args, &grant);
        super::super::TEST_DEFAULT_WORKERS.with(|value| value.set(Some(8)));
        let frozen = super::super::index::execution_options(
            &borrowed(&consuming),
            30,
            super::super::default_workers(),
        );
        super::super::TEST_DEFAULT_WORKERS.with(|value| value.set(Some(4)));
        let rejected = export(&target, &store, &frozen, "rebuild");
        // The rejected eight-worker attempt must leave the original four-worker grant usable.
        let accepted = export(&target, &store, &consuming, "rebuild");
        super::super::TEST_DEFAULT_WORKERS.with(|value| value.set(None));
        assert!(rejected.is_err());
        assert_eq!(
            accepted
                .expect("original budget remains approved")
                .chunks
                .len(),
            1
        );
    }

    #[test]
    fn build_grant_is_consumed_once_and_cannot_authorize_rollback() {
        let (_root, target, store, args) = fixture();
        let grant = issue(&target, &borrowed(&args)).expect("issue");
        let consuming = with_token(&args, &grant);
        assert!(export(&target, &store, &consuming, "rollback").is_err());
        assert_eq!(
            export(&target, &store, &consuming, "rebuild")
                .expect("authorized export")
                .chunks
                .len(),
            1
        );
        assert!(export(&target, &store, &consuming, "rebuild").is_err());
    }

    #[test]
    fn query_grant_does_not_authorize_build_and_remains_usable_for_its_query() {
        let (_root, target, store, args) = fixture();
        let mut command = vec![
            "--user-root".to_owned(),
            target.files.root_path().to_string_lossy().into_owned(),
            "--collection".to_owned(),
            "user-root".to_owned(),
            "--query".to_owned(),
            "synthetic".to_owned(),
            "--scope".to_owned(),
            "collection:user-root".to_owned(),
            "--confirm-current-action".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        for (key, value) in &args {
            command.extend([key.clone(), value.clone()]);
        }
        let grant = knowledge::run_authorize_confidential(&command)
            .expect("query grant")
            .data
            .expect("data");
        let consuming = with_token(&args, &grant);
        assert!(export(&target, &store, &consuming, "rebuild").is_err());
        let mut query = vec![
            "--user-root".to_owned(),
            target.files.root_path().to_string_lossy().into_owned(),
            "--query".to_owned(),
            "synthetic".to_owned(),
            "--scope".to_owned(),
            "collection:user-root".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        for (key, value) in &consuming {
            if !["--nonce", "--expires-at"].contains(&key.as_str()) {
                query.extend([key.clone(), value.clone()]);
            }
        }
        assert_eq!(
            knowledge::run_retrieve(&query)
                .expect("original query authority")
                .data
                .expect("data")["hits"]
                .as_array()
                .expect("hits")
                .len(),
            1
        );
    }

    #[test]
    fn changed_budget_scope_or_capability_cannot_reuse_prior_build_approval() {
        let (_root, target, store, args) = fixture();
        let grant = issue(&target, &borrowed(&args)).expect("issue");
        let consuming = with_token(&args, &grant);
        let mut changed = consuming.clone();
        changed.push((
            "--workers".to_owned(),
            if super::super::default_workers() == 1 {
                "2"
            } else {
                "1"
            }
            .to_owned(),
        ));
        assert!(export(&target, &store, &changed, "rebuild").is_err());
        let (control, digest) = scope_control(&target).expect("control");
        let mut control = control.expect("scope");
        control.revision += 1;
        target
            .files
            .write_control(Some(&target.scope_id), digest.as_deref(), &control)
            .expect("new control epoch");
        assert!(export(&target, &store, &consuming, "rebuild").is_err());
        let mut next = args.clone();
        next.iter_mut()
            .find(|(key, _)| key == "--nonce")
            .expect("nonce")
            .1 = "next-synthetic-approval".to_owned();
        let grant = issue(&target, &borrowed(&next)).expect("fresh issue");
        let capability = Path::new(required(&borrowed(&next), "--capabilities").expect("path"));
        fs::write(capability, b"{\"schema_version\":1,\"changed\":true}")
            .expect("changed capability");
        assert!(export(&target, &store, &with_token(&next, &grant), "rebuild").is_err());
    }

    #[test]
    fn unindexed_canonical_edit_rejects_export_without_consuming_grant() {
        let (_root, target, store, args) = fixture();
        let grant = issue(&target, &borrowed(&args)).expect("issue");
        let snapshot = store
            .load_canonical_snapshot(2)
            .expect("canonical snapshot");
        let claim = target.files.root_path().join(&snapshot.claims[0].locator);
        let bytes = fs::read(&claim).expect("claim bytes");
        fs::write(&claim, b"unindexed canonical edit").expect("edit");
        let consuming = with_token(&args, &grant);
        assert!(export(&target, &store, &consuming, "rebuild").is_err());
        fs::write(&claim, bytes).expect("restore exact fixture");
        assert_eq!(
            export(&target, &store, &consuming, "rebuild")
                .expect("unconsumed grant")
                .chunks
                .len(),
            1
        );
    }

    #[test]
    fn invalid_confidential_yaml_cannot_leak_parser_values_before_approval() {
        let (_root, target, store, args) = fixture();
        let grant = issue(&target, &borrowed(&args)).expect("issue");
        let snapshot = store.load_canonical_snapshot(2).expect("snapshot");
        let locator = &snapshot.claims[0].locator;
        let claim = target.files.root_path().join(locator);
        let original = fs::read_to_string(&claim).expect("claim");
        let invalid = original.replace("kind: decision", "kind: PRIVATE-MARKER");
        assert_ne!(invalid, original);
        fs::write(&claim, invalid).expect("invalid private enum");
        let consuming = with_token(&args, &grant);
        let error =
            export(&target, &store, &consuming, "rebuild").expect_err("reject before approval");
        assert!(!error.to_string().contains("PRIVATE-MARKER"));
        assert!(!error.to_string().contains(locator));
        fs::write(claim, original).expect("restore fixture");
        assert_eq!(
            export(&target, &store, &consuming, "rebuild")
                .expect("grant was not consumed")
                .chunks
                .len(),
            1
        );
    }

    #[test]
    fn expired_approval_request_never_exports_confidential_content() {
        let (_root, target, _store, mut args) = fixture();
        args.iter_mut()
            .find(|(key, _)| key == "--expires-at")
            .expect("expiry")
            .1 = "1".to_owned();
        assert!(issue(&target, &borrowed(&args)).is_err());
    }
}
