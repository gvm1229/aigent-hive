//! Display notices without changing machine-readable stdout or authority.
use crate::run::PinnedTarget;
use crate::user_setup::{load_operational_config, InterfaceLanguage};
use hive_core::gods::God;
use std::path::Path;
pub(crate) fn decorate(
    value: &mut serde_json::Value,
    god: &God,
    user_root: &Path,
) -> Result<(), String> {
    let root = PinnedTarget::open_usage(user_root).map_err(|e| e.message().to_owned())?;
    let config = load_operational_config(root.target_dir())
        .map_err(|e| format!("invalid Hive language setting: {e:?}"))?;
    let korean = config.is_some_and(|c| c.interface_language == InterfaceLanguage::Ko);
    let announcement = god.announcement(korean);
    let prefix = god.prefix();
    value["god"] = god.name.into();
    value["display_prefix"] = prefix.clone().into();
    value["announcement"] = announcement.clone().into();
    if !crate::display_options::quiet() {
        eprintln!("{prefix} {announcement}");
    }
    Ok(())
}

pub(crate) fn run_alias(god: &God, args: &[String]) -> std::process::ExitCode {
    let expanded = match alias_arguments(god, args) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{} {error}", god.prefix());
            return std::process::ExitCode::from(2);
        }
    };
    if expanded.first().is_some_and(|command| command == "summon") {
        return crate::dispatch(expanded);
    }
    let user = args
        .windows(2)
        .find(|pair| pair[0] == "--user-root")
        .map(|pair| std::path::PathBuf::from(&pair[1]))
        .map_or_else(crate::user_install::resolve_user_root_path, Ok);
    match user.and_then(|user| decorate(&mut serde_json::json!({}), god, &user)) {
        Ok(()) => crate::dispatch(expanded),
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::from(2)
        }
    }
}
fn alias_arguments(god: &God, args: &[String]) -> Result<Vec<String>, String> {
    let action = args.first().map_or("--help", String::as_str);
    if action == "--help" {
        return Ok(vec!["summon".into(), god.name.into(), "--help".into()]);
    }
    let mut expanded: Vec<String> = match (god.name, action) {
        (_, "summon") => vec!["summon".into(), god.name.into()],
        ("hades", "plan") => {
            if args[1..]
                .iter()
                .any(|a| a == "--confirm" || a == "--dry-run")
            {
                return Err("hades plan is preview-only; no mode override".into());
            }
            vec!["prune".into(), "--dry-run".into()]
        }
        ("athena" | "poseidon", "plan") => vec!["orchestration".into(), "plan".into()],
        (_, command) if command_owner(command) == Some(god.name) => vec![command.into()],
        _ => return Err(format!("unsupported {} command: {action}", god.name)),
    };
    expanded.extend_from_slice(&args[1..]);
    Ok(expanded)
}
fn command_owner(command: &str) -> Option<&'static str> {
    match command {
        "run" | "loop" | "role" | "agent" => Some("hephaistos"),
        "route" | "doctor" | "check-target" => Some("artemis"),
        "prompt" | "korean" => Some("apollon"),
        "setup" | "install" | "update" | "project" => Some("eileithyia"),
        "orchestration" | "session" | "hook" | "policy" => Some("poseidon"),
        "knowledge" | "source-wiki" | "index" => Some("demeter"),
        "usage" => Some("aither"),
        "storage" | "prune" | "uninstall" => Some("hades"),
        "roster" | "judge" | "report" | "release" | "discord" => Some("athena"),
        _ => None,
    }
}
#[cfg(test)]
mod alias_tests {
    use super::*;
    #[test]
    fn gods_hades_plan_is_an_exact_non_destructive_alias() {
        let god = hive_core::gods::find("하데스").unwrap();
        assert_eq!(
            alias_arguments(god, &["plan".into(), "--output".into(), "json".into()]).unwrap(),
            ["prune", "--dry-run", "--output", "json"]
        );
        assert!(alias_arguments(god, &["plan".into(), "--confirm".into()]).is_err());
        assert!(alias_arguments(god, &["usage".into()]).is_err());
    }
}
