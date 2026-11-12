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
