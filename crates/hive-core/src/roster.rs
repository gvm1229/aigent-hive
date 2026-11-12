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
