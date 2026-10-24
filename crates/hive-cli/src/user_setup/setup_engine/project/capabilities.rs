use crate::user_setup::{SelectedHost, SetupError};
use hive_core::sha256_digest;
use serde_json::{json, Value};

pub(super) fn inspect(host: &str) -> Result<(Value, Value), SetupError> {
    let selected: SelectedHost = serde_json::from_value(json!(host))
        .map_err(|_| SetupError::Input("unknown project host".into()))?;
    let (present, version) = crate::user_install::inspect_setup_host(selected);
    let facts = json!({"executable_present":present,"qualified_installer_version":version,
        "execution_capabilities_verified":false});
    let matrix = unverified(host, version.as_deref().unwrap_or("unverified"))?;
    Ok((matrix, facts))
}

fn unverified(host: &str, version: &str) -> Result<Value, SetupError> {
    let mut matrix = json!({"schema_version":1,"host":host,"host_version":version,"surface":"cli",
        "detection":"unknown","external_runtime":null,"resolved_owner":"host-native",
        "capabilities":{"instructions":"unverified","simple-question-isolation":"unverified",
        "subagents":"unverified","persistent-role-binding":"unverified","continuous-loop":"unverified",
        "usage-sensor":"unverified","independent-judge":"unverified"},
        "evidence":[{"source":"host-catalog","locator":"hive:project-setup:no-runtime-qualification",
        "outcome":"unavailable","digest":sha256_digest(b"no host runtime capability attestation")}]});
    matrix["evidence_digest"] =
        json!(hive_core::preferences::declaration_digest(&matrix).map_err(SetupError::Internal)?);
    hive_core::run::CapabilityResolution::parse_json(
        &serde_json::to_vec(&matrix).map_err(|e| SetupError::Internal(e.to_string()))?,
    )
    .map_err(|error| SetupError::Verification(error.to_string()))?;
    Ok(matrix)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn version_or_presence_never_grants_execution_support_or_an_external_owner() {
        for host in ["codex", "claude", "antigravity"] {
            let matrix = unverified(host, "1.2.3").unwrap();
            assert_eq!(matrix["resolved_owner"], "host-native");
            assert!(matrix["external_runtime"].is_null());
            assert!(matrix["capabilities"]
                .as_object()
                .unwrap()
                .values()
                .all(|value| value == "unverified"));
        }
    }
}
