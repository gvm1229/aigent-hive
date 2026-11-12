//! Strict, reloadable role configuration. No host or credential access.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Workspace {
    pub provider: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Provider {
    pub minimum_version: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Role {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Roster {
    pub workspace: Workspace,
    pub providers: BTreeMap<String, Provider>,
    pub roles: BTreeMap<String, Role>,
    pub gods: BTreeMap<String, String>,
}

const BUILTINS: &[&str] = &[
    include_str!("../../../harness/roles/hive-article-writer.json"),
    include_str!("../../../harness/roles/hive-complex-implementer.json"),
    include_str!("../../../harness/roles/hive-design-specialist.json"),
    include_str!("../../../harness/roles/hive-independent-judge.json"),
    include_str!("../../../harness/roles/hive-research-specialist.json"),
    include_str!("../../../harness/roles/hive-routine-implementer.json"),
];

impl Roster {
    /// Read defaults, then user and project overrides, for every invocation.
    /// # Errors
    /// Rejects malformed TOML, unknown keys, unsafe paths and invalid references.
    pub fn load(user_root: &Path, target: &Path) -> Result<Self, String> {
        let mut roster = Self::defaults()?;
        for root in [user_root, target] {
            let relative = Path::new(".hive/roster.toml");
            crate::ensure_no_symlink_ancestors(root, relative).map_err(|e| e.to_string())?;
            match fs::read_to_string(root.join(relative)) {
                Ok(text) => {
                    roster.merge(toml::from_str(&text).map_err(|e| format!("roster: {e}"))?)
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.to_string()),
            }
        }
        roster.validate()?;
        Ok(roster)
    }

    /// Embedded role defaults; legacy role files remain the initial source.
    /// # Errors
    /// Returns an error for an invalid embedded profile.
    pub fn defaults() -> Result<Self, String> {
        let mut roster = Self {
            workspace: Workspace {
                provider: Some("codex".into()),
            },
            ..Self::default()
        };
        for text in BUILTINS {
            let profile: crate::custom_agent::CustomAgentProfile =
                serde_json::from_str(text).map_err(|e| e.to_string())?;
            let mapping = &profile.host_mappings["codex"];
            roster.roles.insert(
                profile.role_id,
                Role {
                    provider: None,
                    model: Some(mapping.model.clone()),
                    effort: Some(
                        serde_json::to_value(&mapping.effort)
                            .map_err(|e| e.to_string())?
                            .as_str()
                            .ok_or("invalid effort")?
                            .to_owned(),
                    ),
                },
            );
        }
        for provider in ["codex", "claude", "gemini"] {
            roster
                .providers
                .insert(provider.into(), Provider::default());
        }
        Ok(roster)
    }

    fn merge(&mut self, patch: Self) {
        if patch.workspace.provider.is_some() {
            self.workspace = patch.workspace;
        }
        for (name, provider) in patch.providers {
            let current = self.providers.entry(name).or_default();
            if provider.minimum_version.is_some() {
                current.minimum_version = provider.minimum_version;
            }
        }
        for (name, role) in patch.roles {
            let current = self.roles.entry(name).or_default();
            if role.provider.is_some() {
                current.provider = role.provider;
            }
            if role.model.is_some() {
                current.model = role.model;
            }
            if role.effort.is_some() {
                current.effort = role.effort;
            }
        }
        self.gods.extend(patch.gods);
    }

    fn validate(&self) -> Result<(), String> {
        if self
            .providers
            .keys()
            .any(|p| !matches!(p.as_str(), "codex" | "claude" | "gemini"))
        {
            return Err("unknown provider".into());
        }
        for role in self.roles.values() {
            let provider = role
                .provider
                .as_ref()
                .or(self.workspace.provider.as_ref())
                .ok_or("missing provider")?;
            if !self.providers.contains_key(provider)
                || role.model.as_deref().is_none_or(str::is_empty)
                || role.effort.as_deref().is_none_or(str::is_empty)
            {
                return Err("role requires a known provider, exact model and effort".into());
            }
        }
        if self
            .gods
            .values()
            .any(|role| !self.roles.contains_key(role))
        {
            return Err("unknown god role".into());
        }
        Ok(())
    }

    /// Resolve a role or configured god without selecting a replacement model.
    /// # Errors
    /// Rejects missing roles or incomplete configuration.
    pub fn resolve(&self, name: &str) -> Result<(&str, &Role), String> {
        let name = self.gods.get(name).map_or(name, String::as_str);
        let role = self
            .roles
            .get(name)
            .ok_or_else(|| format!("unknown role: {name}"))?;
        let provider = role
            .provider
            .as_deref()
            .or(self.workspace.provider.as_deref())
            .ok_or("missing provider")?;
        Ok((provider, role))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roster_merge_reload_and_defaults() {
        let user = tempfile::tempdir().unwrap();
        let project = tempfile::tempdir().unwrap();
        for root in [user.path(), project.path()] {
            fs::create_dir(root.join(".hive")).unwrap();
        }
        fs::write(
            user.path().join(".hive/roster.toml"),
            "[roles.hive-routine-implementer]\nmodel='exact-user'\neffort='low'\n",
        )
        .unwrap();
        fs::write(
            project.path().join(".hive/roster.toml"),
            "[roles.hive-routine-implementer]\neffort='high'\n",
        )
        .unwrap();
        let roster = Roster::load(user.path(), project.path()).unwrap();
        let (_, role) = roster.resolve("hive-routine-implementer").unwrap();
        assert_eq!(role.model.as_deref(), Some("exact-user"));
        assert_eq!(role.effort.as_deref(), Some("high"));
        assert_eq!(roster.roles.len(), 6);
        fs::write(
            project.path().join(".hive/roster.toml"),
            "[roles.hive-routine-implementer]\neffort='low'\n",
        )
        .unwrap();
        assert_eq!(
            Roster::load(user.path(), project.path())
                .unwrap()
                .resolve("hive-routine-implementer")
                .unwrap()
                .1
                .effort
                .as_deref(),
            Some("low")
        );
    }
    #[test]
    fn roster_rejects_unknown_keys_and_references() {
        for text in [
            "other=true",
            "[workspace]\nother=true",
            "[roles.x]\nother=true",
            "[providers.codex]\nother=true",
        ] {
            assert!(toml::from_str::<Roster>(text).is_err());
        }
        let mut roster = Roster::defaults().unwrap();
        roster.gods.insert("hades".into(), "missing".into());
        assert!(roster.validate().is_err());
    }
}

/// Host-supplied official output, not an execution or activation authority.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityReport {
    pub provider: String,
    pub version: String,
    pub version_output: String,
    pub help_output: String,
    pub login_status_output: String,
    pub models: BTreeMap<String, Vec<String>>,
    pub project_hook_events: Vec<String>,
}

impl Roster {
    /// Check exact role support from explicit host observations. Never launches a process.
    /// # Errors
    /// Rejects unknown versions, models, efforts, login states or missing official hooks.
    pub fn check(&self, name: &str, report: &CapabilityReport) -> Result<(), String> {
        let (provider, role) = self.resolve(name)?;
        if report.provider != provider
            || !report
                .version_output
                .split_whitespace()
                .any(|v| v == report.version)
        {
            return Err("provider/version output mismatch".into());
        }
        let version = version_tuple(&report.version).ok_or("unknown installed version")?;
        let minimum = self.providers[provider]
            .minimum_version
            .as_deref()
            .unwrap_or(match provider {
                "codex" => "0.167.2",
                "claude" => "2.1.163",
                "gemini" => "0.35.3",
                _ => return Err("unknown provider".into()),
            });
        if version < version_tuple(minimum).ok_or("invalid minimum version")? {
            return Err("unsupported installed version".into());
        }
        let model = role.model.as_ref().ok_or("missing model")?;
        let effort = role.effort.as_ref().ok_or("missing effort")?;
        if !report
            .models
            .get(model)
            .is_some_and(|efforts| efforts.contains(effort))
        {
            return Err("unsupported exact model/effort; no fallback".into());
        }
        if provider == "gemini" {
            return Err(
                "Gemini subscription status and exact effort control unverified for 0.35.3".into(),
            );
        }
        if provider == "claude" {
            let status: serde_json::Value = serde_json::from_str(&report.login_status_output)
                .map_err(|_| "invalid official Claude auth status JSON")?;
            if status["loggedIn"] != true
                || status["authMethod"] != "claude.ai"
                || !status["subscriptionType"].is_string()
            {
                return Err("Claude subscription login unverified".into());
            }
        }
        let (login, flags, events): (&str, &[&str], &[&str]) = match provider {
            "codex" => (
                "Logged in using ChatGPT",
                &["--model", "--output-last-message", "--json"],
                &["UserPromptSubmit", "Stop"],
            ),
            "claude" => (
                "\"subscriptionType\"",
                &["--model"],
                &["UserPromptSubmit", "Stop"],
            ),
            "gemini" => (
                "Logged in with Google",
                &["--model", "--output-format", "--prompt"],
                &["BeforeAgent", "AfterAgent"],
            ),
            _ => return Err("unknown provider".into()),
        };
        if !report.login_status_output.contains(login)
            || report.login_status_output.contains("Not logged in")
        {
            return Err("subscription login unverified from official status output".into());
        }
        if flags.iter().any(|flag| !report.help_output.contains(flag))
            || events
                .iter()
                .any(|event| !report.project_hook_events.iter().any(|e| e == event))
        {
            return Err("official command or project hook support unverified".into());
        }
        Ok(())
    }
}

fn version_tuple(text: &str) -> Option<(u32, u32, u32)> {
    let parts = text
        .split('.')
        .map(str::parse)
        .collect::<Result<Vec<u32>, _>>()
        .ok()?;
    match parts.as_slice() {
        [major, minor, patch] => Some((*major, *minor, *patch)),
        _ => None,
    }
}

#[cfg(test)]
mod capability_tests {
    use super::*;
    #[test]
    fn roster_official_output_checks_fail_closed() {
        let roster = Roster::defaults().unwrap();
        let role = &roster.roles["hive-routine-implementer"];
        let report = CapabilityReport {
            provider: "codex".into(),
            version: "0.167.2".into(),
            version_output: "codex-cli 0.167.2".into(),
            help_output: "--model --output-last-message --json".into(),
            login_status_output: "Logged in using ChatGPT".into(),
            models: BTreeMap::from([(
                role.model.clone().unwrap(),
                vec![role.effort.clone().unwrap()],
            )]),
            project_hook_events: vec!["Stop".into(), "UserPromptSubmit".into()],
        };
        roster.check("hive-routine-implementer", &report).unwrap();
        let mut bad = report.clone();
        bad.models.clear();
        assert!(roster.check("hive-routine-implementer", &bad).is_err());
        let mut bad = report.clone();
        bad.models.values_mut().next().unwrap().clear();
        assert!(roster.check("hive-routine-implementer", &bad).is_err());
        let mut bad = report.clone();
        bad.project_hook_events.clear();
        assert!(roster.check("hive-routine-implementer", &bad).is_err());
        let mut bad = report.clone();
        bad.login_status_output = "Not logged in".into();
        assert!(roster.check("hive-routine-implementer", &bad).is_err());
        let mut bad = report;
        bad.version = "0.100.0".into();
        assert!(roster.check("hive-routine-implementer", &bad).is_err());
    }
}

/// Declarative host instructions. Hive never executes these arguments.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerProfile {
    pub provider: String,
    pub model: String,
    pub effort: String,
    pub argv: Vec<String>,
    pub input_mode: String,
    pub result_format: String,
    pub supported: bool,
    pub guidance: String,
}

impl Roster {
    /// Prepare a profile without a model call or implicit fallback.
    /// # Errors
    /// Rejects unknown roles, floating models and unsupported effort values.
    pub fn worker_profile(&self, name: &str) -> Result<WorkerProfile, String> {
        let (provider, role) = self.resolve(name)?;
        let model = role.model.clone().ok_or("missing model")?;
        let effort = role.effort.clone().ok_or("missing effort")?;
        if model.is_empty()
            || matches!(model.as_str(), "auto" | "default" | "latest")
            || model.chars().any(char::is_control)
        {
            return Err("exact model required".into());
        }
        let mut profile = WorkerProfile {
            provider: provider.into(),
            model: model.clone(),
            effort: effort.clone(),
            argv: Vec::new(),
            input_mode: "interactive-role-guidance".into(),
            result_format: "host-receipt".into(),
            supported: true,
            guidance: String::new(),
        };
        match provider {
            "codex" => {
                if !matches!(effort.as_str(), "low" | "medium" | "high" | "xhigh" | "max") {
                    return Err("unsupported Codex effort".into());
                }
                profile.argv = [
                    "codex".into(),
                    "exec".into(),
                    "--model".into(),
                    model,
                    "-c".into(),
                    format!("model_reasoning_effort=\"{effort}\""),
                    "-c".into(),
                    "service_tier=\"priority\"".into(),
                    "-c".into(),
                    "forced_login_method=\"chatgpt\"".into(),
                    "--sandbox".into(),
                    "read-only".into(),
                    "--json".into(),
                    "-".into(),
                ]
                .into();
                profile.input_mode = "stdin-task".into();
                profile.result_format = "jsonl".into();
                profile.guidance="Codex CLI 0.167.2: host supplies task on stdin, captures JSONL final agent message and issues a bound HostReceipt. Check exact model/effort and ChatGPT login first.".into();
            }
            "claude" => {
                if !matches!(effort.as_str(), "low" | "medium" | "high" | "max") {
                    return Err("unsupported Claude effort".into());
                }
                profile.guidance=format!("Use the subscribed interactive Claude host. Select model {model} and effort {effort} in that host. Follow its official hooks and issue bound receipts; no non-interactive Claude invocation.");
            }
            "gemini" => {
                profile.supported = false;
                profile.input_mode = "stdin-task".into();
                profile.result_format = "json".into();
                profile.guidance=format!("Gemini CLI 0.35.3 documents --model {model}, --prompt and --output-format json (response field). Exact effort {effort} and official subscription status are unverified; no executable profile or fallback. Host qualification required.");
            }
            _ => return Err("unknown provider".into()),
        }
        Ok(profile)
    }
}

#[cfg(test)]
mod worker_tests {
    use super::*;
    #[test]
    fn roster_four_role_profiles_and_provider_boundaries() {
        let mut roster = Roster::defaults().unwrap();
        for name in [
            "hive-routine-implementer",
            "hive-complex-implementer",
            "hive-design-specialist",
            "hive-independent-judge",
        ] {
            let p = roster.worker_profile(name).unwrap();
            let role = &roster.roles[name];
            assert_eq!(
                p.argv[0..4],
                ["codex", "exec", "--model", role.model.as_deref().unwrap()]
            );
            assert_eq!(
                p.argv[5],
                format!(
                    "model_reasoning_effort=\"{}\"",
                    role.effort.as_deref().unwrap()
                )
            );
            assert_eq!(p.result_format, "jsonl");
            assert!(p.supported);
            assert_eq!(
                serde_json::from_str::<WorkerProfile>(&serde_json::to_string(&p).unwrap()).unwrap(),
                p
            );
        }
        let role = roster.roles.get_mut("hive-routine-implementer").unwrap();
        role.provider = Some("claude".into());
        let p = roster.worker_profile("hive-routine-implementer").unwrap();
        assert!(p.argv.is_empty());
        assert_eq!(p.input_mode, "interactive-role-guidance");
        roster
            .roles
            .get_mut("hive-routine-implementer")
            .unwrap()
            .provider = Some("gemini".into());
        let p = roster.worker_profile("hive-routine-implementer").unwrap();
        assert!(!p.supported);
        assert!(p.argv.is_empty());
        roster
            .roles
            .get_mut("hive-routine-implementer")
            .unwrap()
            .model = Some("auto".into());
        assert!(roster.worker_profile("hive-routine-implementer").is_err());
    }
}
