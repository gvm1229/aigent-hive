//! Display notices without changing machine-readable stdout or authority.
use crate::run::PinnedTarget;
use crate::user_setup::{load_operational_config, InterfaceLanguage};
use hive_core::gods::{command_owner, God};
use std::path::Path;
pub(crate) fn decorate(
    value: &mut serde_json::Value,
    god: &God,
    user_root: &Path,
) -> Result<(), String> {
    let root = PinnedTarget::open_usage(user_root).map_err(|e| e.message().to_owned())?;
    // A display preference must not prevent an otherwise valid command.
    let config = load_operational_config(root.target_dir()).unwrap_or_default();
    let korean = config.is_some_and(|c| c.interface_language == InterfaceLanguage::Ko);
    let announcement = god.announcement(korean);
    let prefix = god.prefix();
    let notice = if value.get("action").is_some() {
        &mut value["data"]
    } else {
        value
    };
    notice["god"] = god.name.into();
    notice["display_prefix"] = prefix.clone().into();
    notice["announcement"] = announcement.clone().into();
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
#[cfg(test)]
mod alias_tests {
    use super::*;
    #[test]
    fn gods_invalid_language_config_uses_english_without_blocking_aliases() {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join(".hive/config");
        std::fs::create_dir_all(&config).unwrap();
        std::fs::write(
            config.join("user-setup.yml"),
            b"interface_language: [broken",
        )
        .unwrap();
        let god = hive_core::gods::find("hades").unwrap();
        let mut value = serde_json::json!({"data":{}});
        decorate(&mut value, god, root.path()).unwrap();
        assert_eq!(value["announcement"], "Calling upon Hades — cleanup");
        assert_eq!(
            alias_arguments(god, &["plan".into()]).unwrap(),
            ["prune", "--dry-run"]
        );
    }

    #[test]
    fn gods_prepared_notice_keeps_the_existing_action_result_schema() {
        let root = tempfile::tempdir().unwrap();
        let mut value = serde_json::json!({"schema_version":1,"action":"OrchestrationPlan","status":"success","exit_code":0,"code":"hive.orchestration-committed","message":"prepared","changed_paths":[],"evidence":[],"next_action":null,"data":{"state":"prepared"}});
        let mut invalid = value.clone();
        invalid["god"] = "hades".into();
        assert!(hive_core::validate_json_schema(
            include_str!("../../../schemas/action-result.schema.json"),
            &invalid,
            "action result"
        )
        .is_err());
        decorate(
            &mut value,
            hive_core::gods::find("hades").unwrap(),
            root.path(),
        )
        .unwrap();
        hive_core::validate_json_schema(
            include_str!("../../../schemas/action-result.schema.json"),
            &value,
            "action result",
        )
        .unwrap();
        assert_eq!(value["data"]["god"], "hades");
        assert!(value.get("god").is_none());
    }

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
