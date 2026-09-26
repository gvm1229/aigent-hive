//! Internal host state implementation.

use super::{
    env, expected_host_state_after, insert_regular_tree_file, persist_backup, probe_host_snapshot,
    read_optional_regular, read_optional_regular_tree, sha256_digest, AntigravityHostState,
    AntigravityPluginState, ClaudeHostState, ClaudeMarketplaceState, ClaudePluginState,
    CodexHostState, CodexMarketplaceState, CodexPluginState, CommandRunner, HostMutation,
    HostStateSnapshot, HostTransitionPhase, InstallError, Path, PendingHostTransition,
    QualifiedExecutable, RegularTree, UserArguments, UserBackupManifest, UserHost, UserPlan,
    ANTIGRAVITY_SOURCE_RELATIVE, ANTIGRAVITY_STAGE_RELATIVE, COMMAND_OUTPUT_LIMIT, COMMAND_TIMEOUT,
    MAX_USER_FILE_BYTES,
};

pub(super) fn codex_compensation_command(mutation: HostMutation) -> &'static [&'static str] {
    match mutation {
        HostMutation::CodexMarketplaceAdded => {
            &["plugin", "marketplace", "remove", "aigent-hive", "--json"]
        }
        HostMutation::CodexPluginAdded => {
            &["plugin", "remove", "aigent-hive@aigent-hive", "--json"]
        }
        HostMutation::CodexPluginRefreshed => {
            &["plugin", "add", "aigent-hive@aigent-hive", "--json"]
        }
        HostMutation::ClaudeMarketplaceAdded
        | HostMutation::ClaudePluginInstalled
        | HostMutation::ClaudeMarketplaceRefreshed
        | HostMutation::ClaudePluginRefreshed
        | HostMutation::AntigravityPluginInstalled
        | HostMutation::AntigravityPluginRefreshed => unreachable!("Codex mutation required"),
    }
}

pub(super) fn probe_codex_state_if_required(
    arguments: &UserArguments,
    executable: Option<&QualifiedExecutable>,
    runner: &impl CommandRunner,
) -> Result<Option<CodexHostState>, InstallError> {
    if arguments.host != UserHost::Codex {
        return Ok(None);
    }
    let executable = executable.ok_or_else(|| {
        InstallError::Internal("qualified Codex executable is missing".to_owned())
    })?;
    probe_codex_state(executable, runner).map(Some)
}

pub(super) fn probe_codex_state(
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
) -> Result<CodexHostState, InstallError> {
    let marketplace_command = ["plugin", "marketplace", "list", "--json"];
    let plugin_command = ["plugin", "list", "--json"];
    let marketplaces = run_codex_probe(executable, &marketplace_command, runner)?;
    let plugins = run_codex_probe(executable, &plugin_command, runner)?;
    Ok(CodexHostState {
        marketplace: parse_codex_marketplace_state(&marketplaces)?,
        plugin: parse_codex_plugin_state(&plugins)?,
    })
}

pub(super) fn run_codex_probe(
    executable: &QualifiedExecutable,
    command: &[&str],
    runner: &impl CommandRunner,
) -> Result<Vec<u8>, InstallError> {
    let output = runner
        .run(executable, command, COMMAND_TIMEOUT, COMMAND_OUTPUT_LIMIT)
        .map_err(|error| {
            InstallError::Unsupported(format!(
                "Codex structured state probe `{}` failed: {error}",
                command.join(" ")
            ))
        })?;
    if !output.success {
        return Err(InstallError::Unsupported(format!(
            "Codex structured state probe exited unsuccessfully: {}",
            sanitized_command_diagnostic(command, &output.stdout)
        )));
    }
    Ok(output.stdout)
}

pub(super) fn probe_claude_state_if_required(
    arguments: &UserArguments,
    executable: Option<&QualifiedExecutable>,
    runner: &impl CommandRunner,
) -> Result<Option<ClaudeHostState>, InstallError> {
    if arguments.host != UserHost::Claude {
        return Ok(None);
    }
    let executable = executable.ok_or_else(|| {
        InstallError::Internal("qualified Claude executable is missing".to_owned())
    })?;
    probe_claude_state(executable, runner).map(Some)
}

pub(super) fn probe_claude_state(
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
) -> Result<ClaudeHostState, InstallError> {
    let marketplace_command = ["plugin", "marketplace", "list", "--json"];
    let plugin_command = ["plugin", "list", "--json"];
    let marketplaces = run_claude_probe(executable, &marketplace_command, runner)?;
    let plugins = run_claude_probe(executable, &plugin_command, runner)?;
    Ok(ClaudeHostState {
        marketplace: parse_claude_marketplace_state(&marketplaces)?,
        plugin: parse_claude_plugin_state(&plugins)?,
    })
}

pub(super) fn probe_antigravity_state_if_required(
    arguments: &UserArguments,
    executable: Option<&QualifiedExecutable>,
    runner: &impl CommandRunner,
) -> Result<Option<AntigravityHostState>, InstallError> {
    if arguments.host != UserHost::Antigravity {
        return Ok(None);
    }
    let executable = executable.ok_or_else(|| {
        InstallError::Internal("qualified Antigravity executable is missing".to_owned())
    })?;
    probe_antigravity_state(executable, runner).map(Some)
}

pub(super) fn probe_antigravity_state(
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
) -> Result<AntigravityHostState, InstallError> {
    let command = ["plugin", "list"];
    let output = runner
        .run(executable, &command, COMMAND_TIMEOUT, COMMAND_OUTPUT_LIMIT)
        .map_err(|error| {
            InstallError::Unsupported(format!(
                "Antigravity structured state probe `{}` failed: {error}",
                command.join(" ")
            ))
        })?;
    if !output.success {
        return Err(InstallError::Unsupported(format!(
            "Antigravity structured state probe exited unsuccessfully: {}",
            sanitized_command_diagnostic(&command, &output.stdout)
        )));
    }
    parse_antigravity_plugin_state(&output.stdout).map(|plugin| AntigravityHostState { plugin })
}

pub(super) fn parse_antigravity_plugin_state(
    bytes: &[u8],
) -> Result<Option<AntigravityPluginState>, InstallError> {
    let text = std::str::from_utf8(bytes).map_err(|_| {
        InstallError::Verification(
            "Antigravity plugin state probe returned non-UTF-8 output".to_owned(),
        )
    })?;
    if text.trim() == "No imported plugins." {
        return Ok(None);
    }
    let value: serde_json::Value = serde_json::from_str(text).map_err(|_| {
        InstallError::Verification(
            "Antigravity plugin state probe returned malformed structured output".to_owned(),
        )
    })?;
    let entries = value
        .get("imports")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            InstallError::Verification(
                "Antigravity plugin state probe omitted imports[]".to_owned(),
            )
        })?;
    let mut matched = entries.iter().filter(|entry| {
        entry.get("name").and_then(serde_json::Value::as_str) == Some("aigent-hive")
    });
    let first = matched.next();
    if matched.next().is_some() {
        return Err(InstallError::Verification(
            "Antigravity plugin state probe returned duplicate aigent-hive entries".to_owned(),
        ));
    }
    first
        .map(|entry| {
            let source = entry.get("source").and_then(serde_json::Value::as_str);
            let imported_at = entry.get("importedAt").and_then(serde_json::Value::as_str);
            let components = entry
                .get("components")
                .and_then(serde_json::Value::as_array)
                .and_then(|components| {
                    components
                        .iter()
                        .map(serde_json::Value::as_str)
                        .map(|component| component.map(str::to_owned))
                        .collect::<Option<Vec<_>>>()
                });
            match (source, imported_at, components) {
                (Some(source), Some(_), Some(components)) => Ok(AntigravityPluginState {
                    source: source.to_owned(),
                    components,
                }),
                _ => Err(InstallError::Verification(
                    "Antigravity aigent-hive plugin state omitted required fields".to_owned(),
                )),
            }
        })
        .transpose()
}

pub(super) fn expected_antigravity_plugin_state() -> AntigravityPluginState {
    AntigravityPluginState {
        source: "antigravity".to_owned(),
        components: vec!["skills".to_owned()],
    }
}

pub(super) fn run_claude_probe(
    executable: &QualifiedExecutable,
    command: &[&str],
    runner: &impl CommandRunner,
) -> Result<Vec<u8>, InstallError> {
    let output = runner
        .run(executable, command, COMMAND_TIMEOUT, COMMAND_OUTPUT_LIMIT)
        .map_err(|error| {
            InstallError::Unsupported(format!(
                "Claude structured state probe `{}` failed: {error}",
                command.join(" ")
            ))
        })?;
    if !output.success {
        return Err(InstallError::Unsupported(format!(
            "Claude structured state probe exited unsuccessfully: {}",
            sanitized_command_diagnostic(command, &output.stdout)
        )));
    }
    Ok(output.stdout)
}

pub(super) fn parse_claude_marketplace_state(
    bytes: &[u8],
) -> Result<Option<ClaudeMarketplaceState>, InstallError> {
    let entries: Vec<serde_json::Value> = serde_json::from_slice(bytes).map_err(|_| {
        InstallError::Verification(
            "Claude marketplace state probe returned malformed JSON".to_owned(),
        )
    })?;
    let mut matched = entries.iter().filter(|entry| {
        entry.get("name").and_then(serde_json::Value::as_str) == Some("aigent-hive")
    });
    let first = matched.next();
    if matched.next().is_some() {
        return Err(InstallError::Verification(
            "Claude marketplace state probe returned duplicate aigent-hive entries".to_owned(),
        ));
    }
    first
        .map(|entry| {
            let source = entry.get("source").and_then(serde_json::Value::as_str);
            let path = entry.get("path").and_then(serde_json::Value::as_str);
            match (source, path) {
                (Some(source), Some(path)) => Ok(ClaudeMarketplaceState {
                    source: source.to_owned(),
                    path: normalize_host_path(path),
                }),
                _ => Err(InstallError::Verification(
                    "Claude aigent-hive marketplace state omitted source or path".to_owned(),
                )),
            }
        })
        .transpose()
}

pub(super) fn parse_claude_plugin_state(
    bytes: &[u8],
) -> Result<Option<ClaudePluginState>, InstallError> {
    let entries: Vec<serde_json::Value> = serde_json::from_slice(bytes).map_err(|_| {
        InstallError::Verification("Claude plugin state probe returned malformed JSON".to_owned())
    })?;
    let mut matched = entries.iter().filter(|entry| {
        entry.get("id").and_then(serde_json::Value::as_str) == Some("aigent-hive@aigent-hive")
    });
    let first = matched.next();
    if matched.next().is_some() {
        return Err(InstallError::Verification(
            "Claude plugin state probe returned duplicate aigent-hive entries".to_owned(),
        ));
    }
    first
        .map(|entry| {
            let version = entry.get("version").and_then(serde_json::Value::as_str);
            let enabled = entry.get("enabled").and_then(serde_json::Value::as_bool);
            let scope = entry.get("scope").and_then(serde_json::Value::as_str);
            match (version, enabled, scope) {
                (Some(version), Some(enabled), Some(scope)) => Ok(ClaudePluginState {
                    version: version.to_owned(),
                    enabled,
                    scope: scope.to_owned(),
                }),
                _ => Err(InstallError::Verification(
                    "Claude aigent-hive plugin state omitted version, enabled, or scope".to_owned(),
                )),
            }
        })
        .transpose()
}

pub(super) fn parse_codex_marketplace_state(
    bytes: &[u8],
) -> Result<Option<CodexMarketplaceState>, InstallError> {
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| {
        InstallError::Verification(
            "Codex marketplace state probe returned malformed JSON".to_owned(),
        )
    })?;
    let entries = value
        .get("marketplaces")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            InstallError::Verification(
                "Codex marketplace state probe omitted marketplaces[]".to_owned(),
            )
        })?;
    let mut matched = entries.iter().filter(|entry| {
        entry.get("name").and_then(serde_json::Value::as_str) == Some("aigent-hive")
    });
    let first = matched.next();
    if matched.next().is_some() {
        return Err(InstallError::Verification(
            "Codex marketplace state probe returned duplicate aigent-hive entries".to_owned(),
        ));
    }
    first
        .map(|entry| {
            entry
                .get("root")
                .and_then(serde_json::Value::as_str)
                .map(|root| CodexMarketplaceState {
                    root: normalize_host_path(root),
                })
                .ok_or_else(|| {
                    InstallError::Verification(
                        "Codex aigent-hive marketplace state omitted root".to_owned(),
                    )
                })
        })
        .transpose()
}

pub(super) fn parse_codex_plugin_state(
    bytes: &[u8],
) -> Result<Option<CodexPluginState>, InstallError> {
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| {
        InstallError::Verification("Codex plugin state probe returned malformed JSON".to_owned())
    })?;
    let entries = value
        .get("installed")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            InstallError::Verification("Codex plugin state probe omitted installed[]".to_owned())
        })?;
    let mut matched = entries.iter().filter(|entry| {
        entry.get("pluginId").and_then(serde_json::Value::as_str) == Some("aigent-hive@aigent-hive")
    });
    let first = matched.next();
    if matched.next().is_some() {
        return Err(InstallError::Verification(
            "Codex plugin state probe returned duplicate aigent-hive entries".to_owned(),
        ));
    }
    first
        .map(|entry| {
            let version = entry.get("version").and_then(serde_json::Value::as_str);
            let enabled = entry.get("enabled").and_then(serde_json::Value::as_bool);
            let source_path = entry
                .get("source")
                .and_then(|source| source.get("path"))
                .and_then(serde_json::Value::as_str);
            let marketplace_source = entry
                .get("marketplaceSource")
                .and_then(|source| source.get("source"))
                .and_then(serde_json::Value::as_str);
            match (version, enabled, source_path, marketplace_source) {
                (Some(version), Some(enabled), Some(source_path), Some(marketplace_source)) => {
                    Ok(CodexPluginState {
                        version: version.to_owned(),
                        enabled,
                        source_path: normalize_host_path(source_path),
                        marketplace_source: normalize_host_path(marketplace_source),
                    })
                }
                _ => Err(InstallError::Verification(
                    "Codex aigent-hive plugin state omitted required fields".to_owned(),
                )),
            }
        })
        .transpose()
}

pub(super) fn expected_codex_marketplace_root(
    arguments: &UserArguments,
) -> Result<String, InstallError> {
    arguments
        .user_root
        .join(".hive")
        .join("marketplaces")
        .join("codex")
        .to_str()
        .map(normalize_host_path)
        .ok_or_else(|| {
            InstallError::Unsupported("Codex marketplace path is not valid UTF-8".to_owned())
        })
}

pub(super) fn expected_codex_plugin_source_path(
    arguments: &UserArguments,
) -> Result<String, InstallError> {
    arguments
        .user_root
        .join(".hive")
        .join("marketplaces")
        .join("codex")
        .join("plugins")
        .join("aigent-hive")
        .to_str()
        .map(normalize_host_path)
        .ok_or_else(|| {
            InstallError::Unsupported("Codex plugin source path is not valid UTF-8".to_owned())
        })
}

pub(super) fn expected_claude_marketplace_path(
    arguments: &UserArguments,
) -> Result<String, InstallError> {
    arguments
        .user_root
        .join(".hive")
        .join("marketplaces")
        .join("claude")
        .to_str()
        .map(normalize_host_path)
        .ok_or_else(|| {
            InstallError::Unsupported("Claude marketplace path is not valid UTF-8".to_owned())
        })
}

pub(super) fn normalize_host_path(path: &str) -> String {
    #[cfg(windows)]
    {
        let normalized = path.replace('/', "\\");
        if let Some(rest) = normalized.strip_prefix(r"\\?\UNC\") {
            format!(r"\\{rest}")
        } else if let Some(rest) = normalized.strip_prefix(r"\\?\") {
            rest.to_owned()
        } else {
            normalized
        }
    }
    #[cfg(not(windows))]
    {
        path.to_owned()
    }
}

pub(super) fn expected_antigravity_source_path(
    arguments: &UserArguments,
) -> Result<String, InstallError> {
    arguments
        .user_root
        .join(ANTIGRAVITY_SOURCE_RELATIVE)
        .to_str()
        .map(str::to_owned)
        .ok_or_else(|| {
            InstallError::Unsupported(
                "Antigravity plugin source path is not valid UTF-8".to_owned(),
            )
        })
}

pub(super) fn validate_codex_prestate(
    arguments: &UserArguments,
    state: Option<&CodexHostState>,
) -> Result<(), InstallError> {
    let Some(state) = state else {
        return Ok(());
    };
    let expected_marketplace = expected_codex_marketplace_root(arguments)?;
    if state.plugin.is_some() && state.marketplace.is_none() {
        return Err(InstallError::Conflict(
            "Codex aigent-hive plugin exists without its structured marketplace state".to_owned(),
        ));
    }
    if state
        .marketplace
        .as_ref()
        .is_some_and(|marketplace| marketplace.root != expected_marketplace)
    {
        return Err(InstallError::Conflict(
            "Codex aigent-hive marketplace is bound to an unexpected root".to_owned(),
        ));
    }
    if state
        .plugin
        .as_ref()
        .is_some_and(|plugin| plugin.marketplace_source != expected_marketplace || !plugin.enabled)
    {
        return Err(InstallError::Conflict(
            "Codex aigent-hive plugin source or enabled state cannot be restored exactly"
                .to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_codex_activation(
    arguments: &UserArguments,
    executable: Option<&QualifiedExecutable>,
    runner: &impl CommandRunner,
) -> Result<(), InstallError> {
    if arguments.host != UserHost::Codex {
        return Ok(());
    }
    let executable = executable.ok_or_else(|| {
        InstallError::Internal("qualified Codex executable is missing".to_owned())
    })?;
    let state = probe_codex_state(executable, runner)?;
    let marketplace_root = expected_codex_marketplace_root(arguments)?;
    let plugin_source = expected_codex_plugin_source_path(arguments)?;
    let marketplace_valid = state
        .marketplace
        .as_ref()
        .is_some_and(|marketplace| marketplace.root == marketplace_root);
    let plugin_valid = state.plugin.as_ref().is_some_and(|plugin| {
        plugin.version == env!("CARGO_PKG_VERSION")
            && plugin.enabled
            && plugin.source_path == plugin_source
            && plugin.marketplace_source == marketplace_root
    });
    if marketplace_valid && plugin_valid {
        Ok(())
    } else {
        Err(InstallError::Verification(
            "Codex structured state probe did not confirm activated aigent-hive state".to_owned(),
        ))
    }
}

pub(super) fn validate_claude_prestate(
    arguments: &UserArguments,
    state: Option<&ClaudeHostState>,
) -> Result<(), InstallError> {
    let Some(state) = state else {
        return Ok(());
    };
    let expected_path = expected_claude_marketplace_path(arguments)?;
    if state.plugin.is_some() && state.marketplace.is_none() {
        return Err(InstallError::Conflict(
            "Claude aigent-hive plugin exists without its structured marketplace state".to_owned(),
        ));
    }
    if state.marketplace.as_ref().is_some_and(|marketplace| {
        marketplace.source != "directory" || marketplace.path != expected_path
    }) {
        return Err(InstallError::Conflict(
            "Claude aigent-hive marketplace is bound to an unexpected source".to_owned(),
        ));
    }
    if state
        .plugin
        .as_ref()
        .is_some_and(|plugin| plugin.scope != "user")
    {
        return Err(InstallError::Conflict(
            "Claude aigent-hive plugin is not installed at user scope".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_claude_activation(
    arguments: &UserArguments,
    executable: Option<&QualifiedExecutable>,
    runner: &impl CommandRunner,
) -> Result<(), InstallError> {
    if arguments.host != UserHost::Claude {
        return Ok(());
    }
    let executable = executable.ok_or_else(|| {
        InstallError::Internal("qualified Claude executable is missing".to_owned())
    })?;
    let state = probe_claude_state(executable, runner)?;
    let expected_path = expected_claude_marketplace_path(arguments)?;
    let marketplace_valid = state.marketplace.as_ref().is_some_and(|marketplace| {
        marketplace.source == "directory" && marketplace.path == expected_path
    });
    let plugin_valid = state.plugin.as_ref().is_some_and(|plugin| {
        plugin.version == env!("CARGO_PKG_VERSION") && plugin.enabled && plugin.scope == "user"
    });
    if marketplace_valid && plugin_valid {
        Ok(())
    } else {
        Err(InstallError::Verification(
            "Claude structured state probe did not confirm activated aigent-hive state".to_owned(),
        ))
    }
}

pub(super) fn validate_antigravity_prestate(
    arguments: &UserArguments,
    plan: &UserPlan,
    state: Option<&AntigravityHostState>,
) -> Result<(), InstallError> {
    if arguments.host != UserHost::Antigravity {
        return Ok(());
    }
    let plugin = state.and_then(|state| state.plugin.as_ref());
    if plugin.is_some_and(|plugin| plugin != &expected_antigravity_plugin_state()) {
        return Err(InstallError::Conflict(
            "Antigravity aigent-hive plugin has an unsupported native registration shape"
                .to_owned(),
        ));
    }
    let observed_stage =
        read_optional_regular_tree(&arguments.root_cap, Path::new(ANTIGRAVITY_STAGE_RELATIVE))?;
    match (&plan.expected_antigravity_stage, &observed_stage) {
        (None, None) if plugin.is_none() => Ok(()),
        (None, _) => Err(InstallError::Conflict(
            "Antigravity aigent-hive namespace is occupied without authenticated Hive ownership"
                .to_owned(),
        )),
        (Some(expected), Some(observed)) if expected == observed => {
            if plugin.is_some() && !plan.prior_antigravity_activation_source {
                return Err(InstallError::Conflict(
                    "Antigravity aigent-hive plugin is registered without an authenticated Hive source bundle"
                        .to_owned(),
                ));
            }
            Ok(())
        }
        (Some(expected), observed)
            if plugin.is_none()
                && observed.as_ref().is_none_or(|observed| {
                    observed.files.is_empty()
                        && observed.directories.is_subset(&expected.directories)
                }) =>
        {
            Ok(())
        }
        (Some(_), None) => Err(InstallError::Conflict(
            "Antigravity aigent-hive plugin is registered without its authenticated host stage"
                .to_owned(),
        )),
        (Some(_), Some(_)) => Err(InstallError::Conflict(
            "Antigravity aigent-hive host stage differs from its authenticated prior package"
                .to_owned(),
        )),
    }
}

pub(super) fn validate_installed_host(
    arguments: &UserArguments,
    plan: &UserPlan,
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
) -> Result<(), InstallError> {
    match arguments.host {
        UserHost::Codex => validate_codex_activation(arguments, Some(executable), runner),
        UserHost::Claude => validate_claude_activation(arguments, Some(executable), runner),
        UserHost::Antigravity => {
            validate_antigravity_activation(arguments, plan, Some(executable), runner)
        }
    }
}

pub(super) fn validate_antigravity_activation(
    arguments: &UserArguments,
    plan: &UserPlan,
    executable: Option<&QualifiedExecutable>,
    runner: &impl CommandRunner,
) -> Result<(), InstallError> {
    if arguments.host != UserHost::Antigravity {
        return Ok(());
    }
    let executable = executable.ok_or_else(|| {
        InstallError::Internal("qualified Antigravity executable is missing".to_owned())
    })?;
    let state = probe_antigravity_state(executable, runner)?;
    if state.plugin.as_ref() != Some(&expected_antigravity_plugin_state()) {
        return Err(InstallError::Verification(
            "Antigravity structured state probe did not confirm activated aigent-hive state"
                .to_owned(),
        ));
    }
    validate_antigravity_stage(arguments, plan)
}

pub(super) fn planned_antigravity_stage(plan: &UserPlan) -> RegularTree {
    let source = Path::new(ANTIGRAVITY_SOURCE_RELATIVE);
    let mut expected = RegularTree::default();
    for (path, planned) in &plan.files {
        let Ok(relative) = path.strip_prefix(source) else {
            continue;
        };
        insert_regular_tree_file(&mut expected, relative, planned.bytes.clone());
    }
    expected
}

pub(super) fn validate_antigravity_stage(
    arguments: &UserArguments,
    plan: &UserPlan,
) -> Result<(), InstallError> {
    let expected = planned_antigravity_stage(plan);
    let observed =
        read_optional_regular_tree(&arguments.root_cap, Path::new(ANTIGRAVITY_STAGE_RELATIVE))?
            .ok_or_else(|| {
                InstallError::Verification(
                    "Antigravity native plugin staging directory is missing".to_owned(),
                )
            })?;
    if observed != expected {
        return Err(InstallError::Verification(
            "Antigravity native plugin staging differs from the authenticated source tree"
                .to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn sanitized_command_diagnostic(command: &[&str], stdout: &[u8]) -> String {
    format!(
        "argv=`{}`; output-bytes={}; output-digest={}",
        command.join(" "),
        stdout.len(),
        sha256_digest(stdout)
    )
}

pub(super) fn compensate_host_mutations(
    arguments: &UserArguments,
    backup_relative: &Path,
    backup: &mut UserBackupManifest,
    executable: Option<&QualifiedExecutable>,
    runner: &impl CommandRunner,
    allow_dangling_codex_recovery: bool,
) -> Result<(), InstallError> {
    if backup.pending_host_transition.is_some() {
        let executable = executable.ok_or_else(|| {
            InstallError::Internal("qualified recovery host executable is missing".to_owned())
        })?;
        resolve_pending_host_transition(
            arguments,
            backup_relative,
            backup,
            executable,
            runner,
            allow_dangling_codex_recovery,
        )?;
    }
    if backup.host_mutations.is_empty() {
        return Ok(());
    }
    let executable = executable.ok_or_else(|| {
        InstallError::Internal("qualified recovery host executable is missing".to_owned())
    })?;
    match arguments.host {
        UserHost::Codex => {
            let desired = backup.codex_state_before.clone().ok_or_else(|| {
                InstallError::Verification(
                    "Codex transaction backup omitted the pre-mutation structured state".to_owned(),
                )
            })?;
            reconcile_codex_state(
                arguments,
                backup_relative,
                backup,
                &desired,
                executable,
                runner,
            )?;
        }
        UserHost::Claude => {
            let desired = backup.claude_state_before.clone().ok_or_else(|| {
                InstallError::Verification(
                    "Claude transaction backup omitted the pre-mutation structured state"
                        .to_owned(),
                )
            })?;
            reconcile_claude_state(
                arguments,
                backup_relative,
                backup,
                &desired,
                executable,
                runner,
            )?;
        }
        UserHost::Antigravity => {
            let desired = backup.antigravity_state_before.clone().ok_or_else(|| {
                InstallError::Verification(
                    "Antigravity transaction backup omitted the pre-mutation structured state"
                        .to_owned(),
                )
            })?;
            reconcile_antigravity_state(
                arguments,
                backup_relative,
                backup,
                &desired,
                executable,
                runner,
            )?;
        }
    }
    backup.host_mutations.clear();
    backup.host_owned_state = None;
    backup.pending_host_transition = None;
    backup.codex_plugin_was_latent_before_marketplace_add = false;
    persist_backup(arguments, backup_relative, backup)?;
    Ok(())
}

pub(super) fn resolve_pending_host_transition(
    arguments: &UserArguments,
    backup_relative: &Path,
    backup: &mut UserBackupManifest,
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
    allow_dangling_codex_recovery: bool,
) -> Result<(), InstallError> {
    let pending = backup
        .pending_host_transition
        .as_ref()
        .ok_or_else(|| InstallError::Internal("pending host transition is missing".to_owned()))?;
    if allow_dangling_codex_recovery
        && is_recoverable_dangling_codex_marketplace(arguments, backup)?
    {
        let probe_error = probe_host_snapshot(arguments.host, executable, runner)
            .err()
            .ok_or_else(|| {
                InstallError::Conflict(
                    "pending Codex marketplace transition still has a structured host state; external state was preserved"
                        .to_owned(),
                )
            })?;
        if !is_dangling_codex_marketplace_probe(&probe_error) {
            return Err(InstallError::Conflict(format!(
                "unresolved {:?} host transition {:?} cannot be attributed during recovery; external state was preserved",
                pending.phase, pending.mutation
            )));
        }
        let command = codex_compensation_command(HostMutation::CodexMarketplaceAdded);
        let output = runner
            .run(executable, command, COMMAND_TIMEOUT, COMMAND_OUTPUT_LIMIT)
            .map_err(|error| {
                InstallError::Unsupported(format!(
                    "Codex Hive marketplace recovery command failed: {error}"
                ))
            })?;
        if !output.success {
            return Err(InstallError::Unsupported(format!(
                "Codex Hive marketplace recovery command exited unsuccessfully: {}",
                sanitized_command_diagnostic(command, &output.stdout)
            )));
        }
        let observed = probe_host_snapshot(arguments.host, executable, runner)?;
        if observed != pending.before {
            return Err(InstallError::Conflict(
                "Codex Hive marketplace recovery did not restore the authenticated pre-transaction state; external state was preserved"
                    .to_owned(),
            ));
        }
        backup.host_mutations.clear();
        backup.host_owned_state = None;
        backup.pending_host_transition = None;
        backup.codex_plugin_was_latent_before_marketplace_add = false;
        persist_backup(arguments, backup_relative, backup)?;
        return Ok(());
    }
    Err(InstallError::Conflict(format!(
        "unresolved {:?} host transition {:?} cannot be attributed during recovery; external state was preserved",
        pending.phase, pending.mutation
    )))
}

pub(super) fn is_recoverable_dangling_codex_marketplace(
    arguments: &UserArguments,
    backup: &UserBackupManifest,
) -> Result<bool, InstallError> {
    let Some(pending) = backup.pending_host_transition.as_ref() else {
        return Ok(false);
    };
    if arguments.host != UserHost::Codex
        || pending.phase != HostTransitionPhase::Forward
        || pending.mutation != HostMutation::CodexMarketplaceAdded
        || !backup.host_mutations.is_empty()
        || backup.host_owned_state.is_some()
    {
        return Ok(false);
    }
    let Some(before) = backup.codex_state_before.as_ref() else {
        return Ok(false);
    };
    if before.marketplace.is_some() || before.plugin.is_some() {
        return Ok(false);
    }
    let expected_before = HostStateSnapshot::Codex(before.clone());
    let expected_after = expected_host_state_after(
        arguments,
        HostMutation::CodexMarketplaceAdded,
        &expected_before,
    )?;
    if pending.before != expected_before || pending.after != expected_after {
        return Ok(false);
    }
    let manifest = Path::new(".hive/marketplaces/codex/.agents/plugins/marketplace.json");
    Ok(read_optional_regular(&arguments.root_cap, manifest, MAX_USER_FILE_BYTES)?.is_none())
}

pub(super) fn is_dangling_codex_marketplace_probe(error: &InstallError) -> bool {
    matches!(error, InstallError::Unsupported(message)
        if message.starts_with("Codex structured state probe exited unsuccessfully:")
            && message.contains("argv=`plugin marketplace list --json`"))
}

pub(super) struct CompensationContext<'a, R: CommandRunner> {
    arguments: &'a UserArguments,
    backup_relative: &'a Path,
    backup: &'a mut UserBackupManifest,
    executable: &'a QualifiedExecutable,
    runner: &'a R,
}

pub(super) fn reconcile_codex_state(
    arguments: &UserArguments,
    backup_relative: &Path,
    backup: &mut UserBackupManifest,
    desired: &CodexHostState,
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
) -> Result<(), InstallError> {
    let owned = backup.host_owned_state.clone().ok_or_else(|| {
        InstallError::Verification(
            "confirmed host mutations omitted their exact owned state".to_owned(),
        )
    })?;
    let observed = probe_host_snapshot(arguments.host, executable, runner)?;
    if observed != owned {
        return Err(InstallError::Conflict(
            "host state drifted after Hive's confirmed transition; recovery preserved external state"
                .to_owned(),
        ));
    }
    let HostStateSnapshot::Codex(mut current) = owned else {
        return Err(InstallError::Internal(
            "Codex compensation received non-Codex state".to_owned(),
        ));
    };
    let latent_plugin = backup.codex_plugin_was_latent_before_marketplace_add;
    let mut context = CompensationContext {
        arguments,
        backup_relative,
        backup,
        executable,
        runner,
    };
    if current.marketplace != desired.marketplace {
        if current.plugin.is_some() && !latent_plugin {
            let mut after = current.clone();
            after.plugin = None;
            current = run_codex_reconciliation_step(
                &mut context,
                codex_compensation_command(HostMutation::CodexPluginAdded),
                HostMutation::CodexPluginAdded,
                &current,
                &after,
            )?;
        }
        if current.marketplace.is_some() {
            let mut after = current.clone();
            after.marketplace = None;
            if latent_plugin {
                after.plugin = None;
            }
            current = run_codex_reconciliation_step(
                &mut context,
                codex_compensation_command(HostMutation::CodexMarketplaceAdded),
                HostMutation::CodexMarketplaceAdded,
                &current,
                &after,
            )?;
        }
        if let Some(marketplace) = desired.marketplace.as_ref() {
            let command = [
                "plugin",
                "marketplace",
                "add",
                marketplace.root.as_str(),
                "--json",
            ];
            let mut after = current.clone();
            after.marketplace = Some(marketplace.clone());
            current = run_codex_reconciliation_step(
                &mut context,
                &command,
                HostMutation::CodexMarketplaceAdded,
                &current,
                &after,
            )?;
        }
    }
    if current.plugin != desired.plugin {
        let command = if desired.plugin.is_some() {
            codex_compensation_command(HostMutation::CodexPluginRefreshed)
        } else {
            codex_compensation_command(HostMutation::CodexPluginAdded)
        };
        let mut after = current.clone();
        after.plugin.clone_from(&desired.plugin);
        current = run_codex_reconciliation_step(
            &mut context,
            command,
            HostMutation::CodexPluginRefreshed,
            &current,
            &after,
        )?;
    }
    if current == *desired {
        Ok(())
    } else {
        Err(InstallError::Verification(format!(
            "Codex compensation did not restore the pre-mutation structured state for {}",
            arguments.host.as_str()
        )))
    }
}

pub(super) fn reconcile_claude_state(
    arguments: &UserArguments,
    backup_relative: &Path,
    backup: &mut UserBackupManifest,
    desired: &ClaudeHostState,
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
) -> Result<(), InstallError> {
    let owned = backup.host_owned_state.clone().ok_or_else(|| {
        InstallError::Verification(
            "confirmed host mutations omitted their exact owned state".to_owned(),
        )
    })?;
    let observed = probe_host_snapshot(arguments.host, executable, runner)?;
    if observed != owned {
        return Err(InstallError::Conflict(
            "host state drifted after Hive's confirmed transition; recovery preserved external state"
                .to_owned(),
        ));
    }
    let HostStateSnapshot::Claude(mut current) = owned else {
        return Err(InstallError::Internal(
            "Claude compensation received non-Claude state".to_owned(),
        ));
    };
    let mut context = CompensationContext {
        arguments,
        backup_relative,
        backup,
        executable,
        runner,
    };
    current = reconcile_claude_marketplace(&mut context, current, desired)?;
    if current.plugin != desired.plugin {
        if current.plugin.is_some() {
            let mut after = current.clone();
            after.plugin = None;
            current = run_claude_reconciliation_step(
                &mut context,
                &[
                    "plugin",
                    "uninstall",
                    "aigent-hive@aigent-hive",
                    "--scope",
                    "user",
                ],
                HostMutation::ClaudePluginInstalled,
                &current,
                &after,
            )?;
        }
        if let Some(plugin) = desired.plugin.as_ref() {
            let mut after = current.clone();
            after.plugin = Some(ClaudePluginState {
                enabled: true,
                ..plugin.clone()
            });
            current = run_claude_reconciliation_step(
                &mut context,
                &[
                    "plugin",
                    "install",
                    "aigent-hive@aigent-hive",
                    "--scope",
                    "user",
                ],
                HostMutation::ClaudePluginInstalled,
                &current,
                &after,
            )?;
            if !plugin.enabled {
                let mut after = current.clone();
                after.plugin = Some(plugin.clone());
                current = run_claude_reconciliation_step(
                    &mut context,
                    &[
                        "plugin",
                        "disable",
                        "aigent-hive@aigent-hive",
                        "--scope",
                        "user",
                    ],
                    HostMutation::ClaudePluginRefreshed,
                    &current,
                    &after,
                )?;
            }
        }
    }
    if current == *desired {
        Ok(())
    } else {
        Err(InstallError::Verification(format!(
            "Claude compensation did not restore the pre-mutation structured state for {}",
            arguments.host.as_str()
        )))
    }
}

pub(super) fn reconcile_claude_marketplace(
    context: &mut CompensationContext<'_, impl CommandRunner>,
    mut current: ClaudeHostState,
    desired: &ClaudeHostState,
) -> Result<ClaudeHostState, InstallError> {
    if current.marketplace == desired.marketplace {
        return Ok(current);
    }
    if current.plugin.is_some() {
        let mut after = current.clone();
        after.plugin = None;
        current = run_claude_reconciliation_step(
            context,
            &[
                "plugin",
                "uninstall",
                "aigent-hive@aigent-hive",
                "--scope",
                "user",
            ],
            HostMutation::ClaudePluginInstalled,
            &current,
            &after,
        )?;
    }
    if current.marketplace.is_some() {
        let mut after = current.clone();
        after.marketplace = None;
        current = run_claude_reconciliation_step(
            context,
            &[
                "plugin",
                "marketplace",
                "remove",
                "aigent-hive",
                "--scope",
                "user",
            ],
            HostMutation::ClaudeMarketplaceAdded,
            &current,
            &after,
        )?;
    }
    if let Some(marketplace) = desired.marketplace.as_ref() {
        let command = [
            "plugin",
            "marketplace",
            "add",
            marketplace.path.as_str(),
            "--scope",
            "user",
        ];
        let mut after = current.clone();
        after.marketplace = Some(marketplace.clone());
        current = run_claude_reconciliation_step(
            context,
            &command,
            HostMutation::ClaudeMarketplaceAdded,
            &current,
            &after,
        )?;
    }
    Ok(current)
}

pub(super) fn reconcile_antigravity_state(
    arguments: &UserArguments,
    backup_relative: &Path,
    backup: &mut UserBackupManifest,
    desired: &AntigravityHostState,
    executable: &QualifiedExecutable,
    runner: &impl CommandRunner,
) -> Result<(), InstallError> {
    let owned = backup.host_owned_state.clone().ok_or_else(|| {
        InstallError::Verification(
            "confirmed Antigravity mutation omitted its exact owned state".to_owned(),
        )
    })?;
    let observed = probe_host_snapshot(arguments.host, executable, runner)?;
    if observed != owned {
        return Err(InstallError::Conflict(
            "Antigravity state drifted after Hive's confirmed transition; recovery preserved external state"
                .to_owned(),
        ));
    }
    let HostStateSnapshot::Antigravity(mut current) = owned else {
        return Err(InstallError::Internal(
            "Antigravity compensation received non-Antigravity state".to_owned(),
        ));
    };
    let mut context = CompensationContext {
        arguments,
        backup_relative,
        backup,
        executable,
        runner,
    };
    if desired.plugin.is_none() && current.plugin.is_some() {
        let mut after = current.clone();
        after.plugin = None;
        current = run_antigravity_reconciliation_step(
            &mut context,
            &["plugin", "uninstall", "aigent-hive"],
            HostMutation::AntigravityPluginInstalled,
            &current,
            &after,
        )?;
    } else if desired.plugin.is_some() {
        let source_path = expected_antigravity_source_path(arguments)?;
        let validate_command = ["plugin", "validate", source_path.as_str()];
        let output = runner
            .run(
                executable,
                &validate_command,
                COMMAND_TIMEOUT,
                COMMAND_OUTPUT_LIMIT,
            )
            .map_err(|error| {
                InstallError::Internal(format!(
                    "Antigravity compensation validation failed: {error}"
                ))
            })?;
        if !output.success {
            return Err(InstallError::Internal(format!(
                "Antigravity compensation validation returned a non-success result: {}",
                sanitized_command_diagnostic(&validate_command, &output.stdout)
            )));
        }
        let after = desired.clone();
        current = run_antigravity_reconciliation_step(
            &mut context,
            &["plugin", "install", source_path.as_str()],
            HostMutation::AntigravityPluginRefreshed,
            &current,
            &after,
        )?;
        validate_antigravity_recovered_stage(arguments, context.backup)?;
    }
    if current == *desired {
        Ok(())
    } else {
        Err(InstallError::Verification(
            "Antigravity compensation did not restore the pre-mutation structured state".to_owned(),
        ))
    }
}

pub(super) fn run_antigravity_reconciliation_step(
    context: &mut CompensationContext<'_, impl CommandRunner>,
    command: &[&str],
    mutation: HostMutation,
    before: &AntigravityHostState,
    after: &AntigravityHostState,
) -> Result<AntigravityHostState, InstallError> {
    let before = HostStateSnapshot::Antigravity(before.clone());
    let observed = run_compensation_transition(
        context,
        command,
        mutation,
        &before,
        HostStateSnapshot::Antigravity(after.clone()),
    )?;
    match observed {
        HostStateSnapshot::Antigravity(state) => Ok(state),
        HostStateSnapshot::Codex(_) | HostStateSnapshot::Claude(_) => Err(InstallError::Internal(
            "Antigravity compensation observed another host state".to_owned(),
        )),
    }
}

pub(super) fn validate_antigravity_recovered_stage(
    arguments: &UserArguments,
    backup: &UserBackupManifest,
) -> Result<(), InstallError> {
    let source = Path::new(ANTIGRAVITY_SOURCE_RELATIVE);
    let mut expected = RegularTree::default();
    for entry in &backup.entries {
        if !entry.existed {
            continue;
        }
        let source_path = Path::new(&entry.path);
        let Ok(relative) = source_path.strip_prefix(source) else {
            continue;
        };
        let source_bytes =
            read_optional_regular(&arguments.root_cap, source_path, MAX_USER_FILE_BYTES)?
                .ok_or_else(|| {
                    InstallError::Verification(format!(
                        "restored Antigravity source omitted {}",
                        relative.display()
                    ))
                })?;
        insert_regular_tree_file(&mut expected, relative, source_bytes);
    }
    if expected.files.is_empty() {
        return Err(InstallError::Verification(
            "Antigravity recovery has no authenticated prior source bundle".to_owned(),
        ));
    }
    let observed =
        read_optional_regular_tree(&arguments.root_cap, Path::new(ANTIGRAVITY_STAGE_RELATIVE))?
            .ok_or_else(|| {
                InstallError::Verification(
                    "restored Antigravity staging directory is missing".to_owned(),
                )
            })?;
    if observed != expected {
        return Err(InstallError::Verification(
            "restored Antigravity staging differs from the authenticated prior source tree"
                .to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn run_claude_reconciliation_step(
    context: &mut CompensationContext<'_, impl CommandRunner>,
    command: &[&str],
    mutation: HostMutation,
    before: &ClaudeHostState,
    after: &ClaudeHostState,
) -> Result<ClaudeHostState, InstallError> {
    let before = HostStateSnapshot::Claude(before.clone());
    let observed = run_compensation_transition(
        context,
        command,
        mutation,
        &before,
        HostStateSnapshot::Claude(after.clone()),
    )?;
    match observed {
        HostStateSnapshot::Claude(state) => Ok(state),
        HostStateSnapshot::Codex(_) | HostStateSnapshot::Antigravity(_) => Err(
            InstallError::Internal("Claude compensation observed Codex state".to_owned()),
        ),
    }
}

pub(super) fn run_codex_reconciliation_step(
    context: &mut CompensationContext<'_, impl CommandRunner>,
    command: &[&str],
    mutation: HostMutation,
    before: &CodexHostState,
    after: &CodexHostState,
) -> Result<CodexHostState, InstallError> {
    let before = HostStateSnapshot::Codex(before.clone());
    let observed = run_compensation_transition(
        context,
        command,
        mutation,
        &before,
        HostStateSnapshot::Codex(after.clone()),
    )?;
    match observed {
        HostStateSnapshot::Codex(state) => Ok(state),
        HostStateSnapshot::Claude(_) | HostStateSnapshot::Antigravity(_) => Err(
            InstallError::Internal("Codex compensation observed Claude state".to_owned()),
        ),
    }
}

pub(super) fn run_compensation_transition(
    context: &mut CompensationContext<'_, impl CommandRunner>,
    command: &[&str],
    mutation: HostMutation,
    before: &HostStateSnapshot,
    after: HostStateSnapshot,
) -> Result<HostStateSnapshot, InstallError> {
    let observed_before =
        probe_host_snapshot(context.arguments.host, context.executable, context.runner)?;
    if observed_before != *before || context.backup.host_owned_state.as_ref() != Some(before) {
        return Err(InstallError::Conflict(
            "host state drifted immediately before Hive compensation; external state was preserved"
                .to_owned(),
        ));
    }
    context.backup.pending_host_transition = Some(PendingHostTransition {
        mutation,
        phase: HostTransitionPhase::Compensation,
        before: before.clone(),
        after: after.clone(),
    });
    persist_backup(context.arguments, context.backup_relative, context.backup)?;
    let command_result = context.runner.run(
        context.executable,
        command,
        COMMAND_TIMEOUT,
        COMMAND_OUTPUT_LIMIT,
    );
    let observed_after =
        probe_host_snapshot(context.arguments.host, context.executable, context.runner)?;
    if matches!(&command_result, Ok(output) if output.success) && observed_after == after {
        context.backup.host_owned_state = Some(after.clone());
        context.backup.pending_host_transition = None;
        persist_backup(context.arguments, context.backup_relative, context.backup)?;
        return Ok(after);
    }
    match command_result {
        Ok(output) if output.success => Err(InstallError::Internal(format!(
            "{} reconciliation command did not reach its exact structured target state: {}",
            context.arguments.host.as_str(),
            sanitized_command_diagnostic(command, &output.stdout)
        ))),
        Ok(output) => Err(InstallError::Internal(format!(
            "{} reconciliation command returned a non-success result and remains unresolved: {}",
            context.arguments.host.as_str(),
            sanitized_command_diagnostic(command, &output.stdout)
        ))),
        Err(error) => Err(InstallError::Internal(format!(
            "{} reconciliation command `{}` failed before its exact structured target state was observed: {error}",
            context.arguments.host.as_str(),
            command.join(" ")
        ))),
    }
}
