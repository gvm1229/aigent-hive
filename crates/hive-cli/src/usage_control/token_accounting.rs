use super::{ActionResult, AdapterError, Evidence};
use hive_core::{
    sha256_digest,
    token_accounting::{summarize, Request},
};
use std::io::Read;
use std::path::Path;

pub(super) fn run(arguments: &[String]) -> Result<ActionResult, AdapterError> {
    let mut input = None;
    let mut output = false;
    for pair in arguments.chunks(2) {
        if pair.len() != 2 {
            return Err(AdapterError::Input(
                "token summary requires paired options".to_owned(),
            ));
        }
        match pair[0].as_str() {
            "--request" if input.is_none() => input = Some(pair[1].as_str()),
            "--output" if !output && pair[1] == "json" => output = true,
            _ => {
                return Err(AdapterError::Input(
                    "unsupported or duplicate token summary option".to_owned(),
                ))
            }
        }
    }
    if !output {
        return Err(AdapterError::Input(
            "token summary requires --output json".to_owned(),
        ));
    }
    let path = Path::new(
        input.ok_or_else(|| AdapterError::Input("token summary requires --request".to_owned()))?,
    );
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| AdapterError::Input("cannot inspect token accounting request".to_owned()))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 1024 * 1024 {
        return Err(AdapterError::Input(
            "token accounting request must be a bounded regular file".to_owned(),
        ));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|_| AdapterError::Input("cannot open token accounting request".to_owned()))?
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| AdapterError::Input("cannot read token accounting request".to_owned()))?;
    if bytes.len() > 1024 * 1024 {
        return Err(AdapterError::Input(
            "token accounting request exceeds limit".to_owned(),
        ));
    }
    let request: Request = serde_json::from_value(
        crate::usage::parse_strict_native_json(&bytes)
            .map_err(|_| AdapterError::Input("invalid token accounting JSON".to_owned()))?,
    )
    .map_err(|_| AdapterError::Input("invalid normalized token accounting request".to_owned()))?;
    let report =
        summarize(&request).map_err(|reason| AdapterError::Verification(reason.to_owned()))?;
    Ok(ActionResult {
        schema_version: 1,
        action: "CheckUsage",
        status: "success",
        exit_code: 0,
        code: "hive.token-accounting",
        message:
            "normalized token measurement summarized without enforcing a cap or subscription quota"
                .to_owned(),
        changed_paths: vec![],
        evidence: vec![Evidence {
            kind: "report",
            locator: "token-accounting:normalized".to_owned(),
            digest: sha256_digest(&bytes),
        }],
        next_action: None,
        data: Some(serde_json::to_value(report).map_err(|_| {
            AdapterError::Internal("cannot encode token accounting report".to_owned())
        })?),
    })
}
