//! User-facing removal requires a stable, displayed inventory. Internal repair stays separate.
use super::{
    build_plan, capability_parent, deactivation, emit_action_result, failure, file_permissions,
    host_uninstall_root, io_internal, open_canonical_user_root, owned_path_exists, parse_uninstall,
    portable, read_installed_manifest, read_optional_regular, FilePermissions, InstallError,
    UserArguments, UserHost, UserMode, UserUninstallArguments, UNINSTALL_FILES, UNINSTALL_TREES,
};
use crate::ActionResult;
use cap_fs_ext::DirExt;
use cap_fs_ext::MetadataExt;
use cap_std::fs::Dir;
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::io::{BufRead, IsTerminal, Write};
use std::{
    collections::BTreeMap,
    env, io,
    path::{Path, PathBuf},
    process::ExitCode,
};
mod removal;
pub(super) use removal::complete_helper;

const PRESERVED: &[&str] = &[
    ".hive/knowledge",
    ".hive/history",
    ".hive/config/user-setup.yml",
    ".hive/config/user-feature-answers.yml",
    ".hive/config/user-preferences.json",
    ".hive/config/setup-progress",
    "project-source-files",
    "foreign-content-outside-Hive-markers",
    ".hive/config/projects.yml",
    ".hive/runtime/usage-monitor/config.json",
    ".hive/runtime/usage-monitor/disable.json",
    ".hive/runtime/usage-monitor/control.lock",
    ".hive/runtime/usage-monitor/controller.lock",
];

struct Request {
    target: UserUninstallArguments,
    dry_run: bool,
    confirmed: bool,
    json: bool,
    resume: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    kind: String,
    digest: Option<String>,
    after_digest: Option<String>,
    permissions: Option<FilePermissions>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Preview {
    root: PathBuf,
    root_identity: (u64, u64),
    entries: BTreeMap<String, Entry>,
    hosts: Vec<UserHost>,
    monitor: Value,
    core: Value,
    native: Vec<deactivation::Plan>,
    preserved: Vec<String>,
}

fn parse(arguments: &[String]) -> Result<Request, InstallError> {
    let (mut dry_run, mut confirmed, mut json) = (false, false, false);
    let mut resume = None;
    let mut filtered = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--dry-run" if !dry_run => dry_run = true,
            "--confirm" if !confirmed => confirmed = true,
            "--resume" if resume.is_none() => {
                let id = arguments
                    .get(index + 1)
                    .filter(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
                    .ok_or_else(|| {
                        InstallError::Input("--resume needs a removal operation ID".into())
                    })?;
                resume = Some(id.clone());
                index += 1;
            }
            "--output" if !json => {
                if arguments.get(index + 1).map(String::as_str) != Some("json") {
                    return Err(InstallError::Input(
                        "uninstall --output must be json".into(),
                    ));
                }
                json = true;
                index += 1;
            }
            "--user-root" => {
                let path = arguments
                    .get(index + 1)
                    .ok_or_else(|| InstallError::Input("missing user root".into()))?;
                filtered.extend([arguments[index].clone(), path.clone()]);
                index += 1;
            }
            option => {
                return Err(InstallError::Input(format!(
                    "unknown or duplicate uninstall option: {option}"
                )))
            }
        }
        index += 1;
    }
    if dry_run && (confirmed || resume.is_some()) {
        return Err(InstallError::Input(
            "--dry-run conflicts with --confirm or --resume".into(),
        ));
    }
    Ok(Request {
        target: parse_uninstall(&filtered)?,
        dry_run,
        confirmed,
        json,
        resume,
    })
}

pub(super) fn run(arguments: &[String]) -> ExitCode {
    let machine = arguments.windows(2).any(|p| p == ["--output", "json"]);
    let mut reader = io::stdin().lock();
    let mut writer = io::stderr().lock();
    let interactive = io::stdin().is_terminal() && io::stderr().is_terminal();
    let result = parse(arguments)
        .and_then(|request| execute(&request, interactive, &mut reader, &mut writer))
        .unwrap_or_else(|error| failure("UninstallHiveUser", &error));
    if machine {
        emit_action_result(&result)
    } else {
        eprintln!("{}", result.message);
        if let Some(next) = &result.next_action {
            eprintln!("{next}");
        }
        ExitCode::from(result.exit_code)
    }
}

fn root_identity(root: &Dir) -> Result<(u64, u64), InstallError> {
    let metadata = root
        .dir_metadata()
        .map_err(|e| InstallError::Conflict(e.to_string()))?;
    Ok((MetadataExt::dev(&metadata), MetadataExt::ino(&metadata)))
}

fn entry(
    root: &Dir,
    path: &Path,
    kind: &'static str,
    entries: &mut BTreeMap<String, Entry>,
) -> Result<(), InstallError> {
    let name = portable(path);
    // Live lock bytes are inaccessible on Windows and are never a deletion target.
    // The tree reader has already rejected links and non-regular entries.
    if matches!(
        name.as_str(),
        ".hive/runtime/usage-monitor/control.lock" | ".hive/runtime/usage-monitor/controller.lock"
    ) {
        return Ok(());
    }
    if entries.len() >= 16_384 {
        return Err(InstallError::Conflict(
            "removal inventory exceeds its entry limit".into(),
        ));
    }
    let Some(bytes) = read_optional_regular(root, path, 128 * 1024 * 1024)? else {
        return Ok(());
    };
    let after = if kind == "marker" {
        Some(sha256_digest(&deactivation::without_marker(&bytes)?))
    } else {
        None
    };
    let volatile = matches!(
        name.as_str(),
        ".hive/runtime/usage-monitor/state.json" | ".hive/runtime/usage-monitor/startup.json"
    );
    let kind = if matches!(
        name.as_str(),
        ".hive/runtime/usage-monitor/config.json" | ".hive/runtime/usage-monitor/disable.json"
    ) {
        "preserved"
    } else {
        kind
    };
    entries.insert(
        name,
        Entry {
            kind: kind.into(),
            digest: (!volatile).then(|| sha256_digest(&bytes)),
            after_digest: after,
            permissions: Some(file_permissions(root, path)?),
        },
    );
    Ok(())
}

fn tree(
    root: &Dir,
    path: &Path,
    entries: &mut BTreeMap<String, Entry>,
) -> Result<(), InstallError> {
    let Some((parent, name)) = capability_parent(root, path, false)? else {
        return Ok(());
    };
    let metadata = match parent.symlink_metadata(&name) {
        Ok(value) => value,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(io_internal("inspect removal inventory", path, e)),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(InstallError::Conflict(format!(
            "removal directory is unsafe: {}",
            path.display()
        )));
    }
    if entries.len() >= 16_384 {
        return Err(InstallError::Conflict(
            "removal inventory exceeds its entry limit".into(),
        ));
    }
    entries.insert(
        portable(path),
        Entry {
            kind: "directory".into(),
            digest: None,
            after_digest: None,
            permissions: None,
        },
    );
    let directory = parent
        .open_dir_nofollow(&name)
        .map_err(|e| io_internal("pin removal directory", path, e))?;
    for child in directory
        .entries()
        .map_err(|e| io_internal("list removal directory", path, e))?
    {
        let child = child.map_err(|e| io_internal("read removal entry", path, e))?;
        let relative = path.join(child.file_name());
        let metadata = directory
            .symlink_metadata(child.file_name())
            .map_err(|e| io_internal("inspect removal entry", &relative, e))?;
        if metadata.file_type().is_symlink() {
            return Err(InstallError::Conflict(format!(
                "removal entry is a symlink: {}",
                relative.display()
            )));
        }
        if metadata.is_dir() {
            tree(root, &relative, entries)?;
        } else if metadata.is_file() {
            entry(root, &relative, "file", entries)?;
        } else {
            return Err(InstallError::Conflict(
                "removal entry is not a regular file or directory".into(),
            ));
        }
    }
    Ok(())
}

fn preview(arguments: &UserUninstallArguments, executable: &Path) -> Result<Preview, InstallError> {
    let root = &arguments.root_cap;
    let mut entries = BTreeMap::new();
    let mut hosts = Vec::new();
    for host in [UserHost::Codex, UserHost::Claude, UserHost::Antigravity] {
        let path = PathBuf::from(format!(".hive/install/{}.json", host.as_str()));
        let manifest = read_installed_manifest(root, &path, host)?;
        if let Some(manifest) = manifest {
            let args = UserArguments {
                host,
                mode: UserMode::DryRun,
                user_root: arguments.user_root.clone(),
                root_cap: root
                    .try_clone()
                    .map_err(|e| InstallError::Internal(e.to_string()))?,
                setup_override: None,
            };
            build_plan(&args)?;
            hosts.push(host);
            for owned in manifest.entries {
                let path = Path::new(&owned.path);
                if path.starts_with(".hive/knowledge") {
                    continue;
                }
                let kind = if owned.ownership == "shared-marker" {
                    "marker"
                } else {
                    "file"
                };
                entry(root, path, kind, &mut entries)?;
            }
        } else if owned_path_exists(root, host_uninstall_root(host))? {
            return Err(InstallError::Conflict(
                "host source has no ownership manifest; preserved".into(),
            ));
        }
    }
    for path in crate::user_setup::verified_uninstall_projection_paths(root)
        .map_err(|e| InstallError::Conflict(e.message().into()))?
    {
        entry(root, &path, "file", &mut entries)?;
    }
    for path in UNINSTALL_TREES {
        tree(root, Path::new(path), &mut entries)?;
    }
    for path in UNINSTALL_FILES
        .iter()
        .filter(|path| **path != ".hive/config/projects.yml")
    {
        entry(root, Path::new(path), "file", &mut entries)?;
    }
    let monitor = crate::usage_monitor::inspect_saved_choice(&arguments.user_root)
        .map_err(|e| InstallError::Conflict(e.message().into()))?;
    let core = crate::update_activation::uninstall_owner_preview(executable)
        .unwrap_or_else(|reason| json!({"owner":"unverified","preserved":true,"reason":reason}));
    let selected = hosts
        .iter()
        .map(|host| match host {
            UserHost::Codex => crate::user_setup::SelectedHost::Codex,
            UserHost::Claude => crate::user_setup::SelectedHost::Claude,
            UserHost::Antigravity => crate::user_setup::SelectedHost::Antigravity,
        })
        .collect::<Vec<_>>();
    let native =
        deactivation::preview(&arguments.user_root, &selected).map_err(InstallError::Conflict)?;
    Ok(Preview {
        root: arguments.user_root.clone(),
        root_identity: root_identity(root)?,
        entries,
        hosts,
        monitor: json!(monitor),
        core,
        native,
        preserved: PRESERVED.iter().map(|s| (*s).to_owned()).collect(),
    })
}

fn digest(preview: &Preview) -> Result<String, InstallError> {
    serde_json::to_vec(preview)
        .map(|bytes| sha256_digest(&bytes))
        .map_err(|e| InstallError::Internal(e.to_string()))
}

fn verify(
    arguments: &UserUninstallArguments,
    executable: &Path,
    expected: &Preview,
) -> Result<(), InstallError> {
    let (user_root, root_cap) = open_canonical_user_root(&arguments.user_root)?;
    let current = preview(
        &UserUninstallArguments {
            user_root,
            root_cap,
        },
        executable,
    )?;
    if digest(&current)? != digest(expected)? {
        return Err(InstallError::Conflict(
            "removal inventory changed after confirmation; nothing was removed".into(),
        ));
    }
    Ok(())
}

fn report(
    preview: &Preview,
    status: &'static str,
    code: &'static str,
    exit_code: u8,
    ko: bool,
) -> Result<ActionResult, InstallError> {
    Ok(ActionResult {
        schema_version: 1,
        action: "UninstallHiveUser",
        status,
        code,
        exit_code,
        message: if ko {
            "제거 대상과 보존 자료 확인. 아직 삭제한 항목은 없습니다."
        } else {
            "Removal and preservation inspected. Nothing has been removed."
        }
        .into(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: Some(
            json!({"preview":preview,"preview_digest":digest(preview)?,"confirmation_required":true}),
        ),
    })
}

fn execute(
    request: &Request,
    interactive: bool,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<ActionResult, InstallError> {
    if let Some(id) = &request.resume {
        return removal::resume(&request.target, id);
    }
    let executable = env::current_exe().map_err(|e| InstallError::Internal(e.to_string()))?;
    let plan = preview(&request.target, &executable)?;
    let language = crate::user_setup::load_operational_config(&request.target.root_cap)
        .map_err(|e| InstallError::Conflict(e.message().into()))?
        .map(|config| config.interface_language);
    let mut ko = language == Some(crate::user_setup::InterfaceLanguage::Ko);
    if !request.json {
        if language.is_none() && interactive {
            writeln!(writer, "언어 / Language: 1 한국어 · 2 English")
                .map_err(|e| InstallError::Internal(e.to_string()))?;
            writer
                .flush()
                .map_err(|e| InstallError::Internal(e.to_string()))?;
            let mut answer = String::new();
            reader
                .read_line(&mut answer)
                .map_err(|e| InstallError::Input(e.to_string()))?;
            match answer.trim() {
                "1" => ko = true,
                "2" => ko = false,
                _ => return report(&plan, "needs-input", "hive.uninstall-needs-input", 2, ko),
            }
        }
        writeln!(writer, "{}", if ko { "제거 대상:" } else { "Remove:" })
            .map_err(|e| InstallError::Internal(e.to_string()))?;
        for (path, entry) in plan
            .entries
            .iter()
            .filter(|(_, entry)| entry.kind != "preserved")
        {
            writeln!(writer, "  {path} ({})", entry.kind)
                .map_err(|e| InstallError::Internal(e.to_string()))?;
        }
        writeln!(
            writer,
            "{} {}",
            if ko { "보존:" } else { "Preserve:" },
            PRESERVED.join(", ")
        )
        .map_err(|e| InstallError::Internal(e.to_string()))?;
        writeln!(
            writer,
            "{}",
            match (ko, plan.monitor["configured"] == true) {
                (true, true) => "먼저 이 사용자 폴더의 Hive 감시를 종료합니다.",
                (true, false) => "등록된 감시가 없습니다.",
                (false, true) => "Stop the owned Hive monitor before removal.",
                (false, false) => "No monitor is registered.",
            }
        )
        .map_err(|e| InstallError::Internal(e.to_string()))?;
        writeln!(
            writer,
            "{} {}",
            if ko {
                "프로그램 본체:"
            } else {
                "Program:"
            },
            plan.core["executable"].as_str().unwrap_or(if ko {
                "설치 주체 확인 못함. 본체 보존"
            } else {
                "owner unverified; program preserved"
            })
        )
        .map_err(|e| InstallError::Internal(e.to_string()))?;
    }
    if request.dry_run {
        return report(&plan, "success", "hive.uninstall-preview", 0, ko);
    }
    if !request.confirmed {
        if !interactive || request.json {
            return report(&plan, "needs-input", "hive.uninstall-needs-input", 2, ko);
        }
        write!(
            writer,
            "{}",
            if ko {
                "이 목록을 제거할까요? [예/아니요, 기본 아니요] "
            } else {
                "Remove this inventory? [yes/no, default no] "
            }
        )
        .map_err(|e| InstallError::Internal(e.to_string()))?;
        writer
            .flush()
            .map_err(|e| InstallError::Internal(e.to_string()))?;
        let mut answer = String::new();
        reader
            .read_line(&mut answer)
            .map_err(|e| InstallError::Input(e.to_string()))?;
        if !matches!(answer.trim().to_lowercase().as_str(), "y" | "yes" | "예") {
            return report(&plan, "success", "hive.uninstall-declined", 0, ko);
        }
    }
    verify(&request.target, &executable, &plan)?;
    removal::start(&request.target, plan, ko)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io::Cursor};

    fn request(root: &Path, flags: &[&str]) -> Request {
        parse(
            &[
                vec!["--user-root".into(), root.to_string_lossy().into_owned()],
                flags.iter().map(|s| (*s).to_owned()).collect(),
            ]
            .concat(),
        )
        .unwrap()
    }

    #[test]
    fn preview_missing_consent_and_terminal_no_or_eof_never_write() {
        for (flags, interactive, input, code) in [
            (
                vec!["--dry-run", "--output", "json"],
                false,
                "yes\n",
                "hive.uninstall-preview",
            ),
            (
                vec!["--output", "json"],
                true,
                "yes\n",
                "hive.uninstall-needs-input",
            ),
            (vec![], false, "yes\n", "hive.uninstall-needs-input"),
            (vec![], true, "1\nno\n", "hive.uninstall-declined"),
            (vec![], true, "2\n", "hive.uninstall-declined"),
        ] {
            let root = tempfile::tempdir().unwrap();
            fs::write(root.path().join("foreign.txt"), b"keep").unwrap();
            let result = execute(
                &request(root.path(), &flags),
                interactive,
                &mut Cursor::new(input),
                &mut Vec::new(),
            )
            .unwrap();
            assert_eq!(result.code, code);
            assert_eq!(result.changed_paths, Vec::<String>::new());
            assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
            assert_eq!(fs::read(root.path().join("foreign.txt")).unwrap(), b"keep");
        }
    }

    #[test]
    fn consent_binds_file_content_new_entries_and_physical_root() {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join(".hive/index");
        fs::create_dir_all(&config).unwrap();
        let file = config.join("derived.db");
        fs::write(&file, b"before").unwrap();
        let req = request(root.path(), &["--dry-run"]);
        let absent = root.path().join("no-program");
        let plan = preview(&req.target, &absent).unwrap();
        verify(&req.target, &absent, &plan).unwrap();
        fs::write(&file, b"after").unwrap();
        assert!(verify(&req.target, &absent, &plan).is_err());
        fs::write(&file, b"before").unwrap();
        fs::write(config.join("new.db"), b"new").unwrap();
        assert!(verify(&req.target, &absent, &plan).is_err());
        let other = tempfile::tempdir().unwrap();
        assert!(verify(&request(other.path(), &[]).target, &absent, &plan).is_err());
        assert_eq!(fs::read(file).unwrap(), b"before");
    }

    #[test]
    fn malformed_owned_projection_and_unsupported_flags_are_refused_without_deletion() {
        let root = tempfile::tempdir().unwrap();
        for flags in [
            vec!["--full"],
            vec!["--purge"],
            vec!["--confirm", "--confirm"],
            vec!["--dry-run", "--confirm"],
        ] {
            let args = [
                vec![
                    "--user-root".into(),
                    root.path().to_string_lossy().into_owned(),
                ],
                flags.into_iter().map(str::to_owned).collect(),
            ]
            .concat();
            assert!(parse(&args).is_err());
        }
        let config = root.path().join(".hive/config");
        fs::create_dir_all(&config).unwrap();
        let manifest = config.join("user-projection.json");
        // The exact ledger path is obtained from the existing owner, not inferred from content.
        fs::write(&manifest, b"unrelated").unwrap();
        let path = root.path().join(".hive/install/user-projection.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"invalid owned ledger").unwrap();
        let req = request(root.path(), &["--dry-run"]);
        assert!(preview(&req.target, &root.path().join("none")).is_err());
        assert_eq!(fs::read(path).unwrap(), b"invalid owned ledger");
    }
}
