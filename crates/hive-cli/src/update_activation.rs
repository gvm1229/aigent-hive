use crate::usage::{CommandRunner, SystemCommandRunner};
use hive_core::sha256_digest;
use hive_update::SemVersion;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::thread;
use std::time::{Duration, Instant};

const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_BINARY_BYTES: u64 = 128 * 1024 * 1024;
const MAX_INSTALLER_BYTES: usize = 1024 * 1024;
const INSTALL_TIMEOUT: Duration = Duration::from_mins(10);
const INSTALL_OUTPUT_LIMIT: usize = 4 * 1024 * 1024;
const DIRECT_HANDOFF_HELPER: &str = ".hive/runtime/update/hive-direct-update-helper.exe";

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum PackageChannel {
    /// Bare `-test` is the default prerelease; a number is only for a
    /// follow-up immutable test publication.
    Test(Option<u64>),
    Stable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UpdateChannel {
    Stable,
    Test,
}

struct UpdateArguments {
    channel: UpdateChannel,
    confirm: bool,
    handoff_executable: Option<PathBuf>,
    user_root: Option<PathBuf>,
}

#[derive(Clone, Copy)]
struct UpdateSelection {
    channel: UpdateChannel,
    confirmed: bool,
}

const UPDATE_USAGE: &str = "\
Update the installed Aigent Hive package and its authenticated user projections.

USAGE:
    hive update [--channel stable|test] [--user-root <absolute-dir>] [--confirm]

The default channel is stable. The test channel requires explicit --channel test.
--confirm accepts the displayed exact update without an interactive terminal.
";

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PackageVersion {
    product: SemVersion,
    channel: PackageChannel,
    exact: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum InstallOwner {
    Npm {
        package_version: PackageVersion,
    },
    Direct {
        package_version: PackageVersion,
        prefix: PathBuf,
    },
}

impl InstallOwner {
    fn package_version(&self) -> &PackageVersion {
        match self {
            Self::Npm { package_version }
            | Self::Direct {
                package_version, ..
            } => package_version,
        }
    }

    const fn label(&self) -> &'static str {
        match self {
            Self::Npm { .. } => "npm",
            Self::Direct { .. } => "direct",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Language {
    En,
    Ko,
}

#[derive(Debug, Eq, PartialEq)]
enum FlowOutcome {
    Current,
    Declined,
    Installed,
}

struct UpdateFlowContext<'a> {
    executable: &'a Path,
    user_root: &'a Path,
    language: Language,
}

trait ProjectionRefresher {
    fn authenticated_hosts(&self, user_root: &Path) -> Result<Vec<String>, String>;

    fn refresh_and_validate(
        &self,
        executable: &Path,
        user_root: &Path,
        hosts: &[String],
    ) -> Result<(), String>;
}

struct LiveProjectionRefresher;

impl ProjectionRefresher for LiveProjectionRefresher {
    fn authenticated_hosts(&self, user_root: &Path) -> Result<Vec<String>, String> {
        crate::user_install::authenticated_saved_projection_hosts(user_root)
    }

    fn refresh_and_validate(
        &self,
        executable: &Path,
        user_root: &Path,
        hosts: &[String],
    ) -> Result<(), String> {
        let executable = executable.to_string_lossy();
        let program = SystemCommandRunner
            .qualify(&executable)
            .map_err(|error| format!("cannot qualify the activated Hive executable: {error}"))?;
        let hosts = hosts.join(",");
        let user_root = user_root.to_string_lossy();
        for mode in ["--apply", "--validate"] {
            let expected_action = projection_refresh_action(mode);
            let output = SystemCommandRunner
                .run(
                    &program,
                    &[
                        "install",
                        "--scope",
                        "user",
                        "--hosts",
                        &hosts,
                        mode,
                        "--user-root",
                        &user_root,
                        "--output",
                        "json",
                    ],
                    INSTALL_TIMEOUT,
                    INSTALL_OUTPUT_LIMIT,
                )
                .map_err(|error| {
                    format!("activated Hive user projection {mode} command failed: {error}")
                })?;
            let result: ChildActionResult =
                serde_json::from_slice(&output.stdout).map_err(|_| {
                    format!("activated Hive user projection {mode} command returned malformed JSON")
                })?;
            if !output.success || !projection_refresh_reported_success(expected_action, &result) {
                return Err(format!(
                    "activated Hive user projection {mode} command did not report success"
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
struct NoopProjectionRefresher;

#[cfg(test)]
impl ProjectionRefresher for NoopProjectionRefresher {
    fn authenticated_hosts(&self, _user_root: &Path) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }

    fn refresh_and_validate(
        &self,
        _executable: &Path,
        _user_root: &Path,
        _hosts: &[String],
    ) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Deserialize)]
struct ChildActionResult {
    action: String,
    status: String,
    exit_code: u8,
}

fn projection_refresh_action(mode: &str) -> &'static str {
    match mode {
        "--apply" => "InstallHiveUser",
        "--validate" => "ValidateHiveUser",
        _ => unreachable!("unsupported user projection refresh mode"),
    }
}

fn projection_refresh_reported_success(expected_action: &str, result: &ChildActionResult) -> bool {
    result.status == "success" && result.exit_code == 0 && result.action == expected_action
}

trait RegistrySource {
    fn fetch(&self) -> Result<Vec<u8>, String>;
}

struct LiveRegistry;

impl RegistrySource for LiveRegistry {
    fn fetch(&self) -> Result<Vec<u8>, String> {
        crate::update_discovery::fetch_registry_metadata()
    }
}

trait Installer {
    fn install(&self, owner: &InstallOwner, target: &PackageVersion) -> Result<(), String>;
}

struct LiveInstaller;

impl Installer for LiveInstaller {
    fn install(&self, owner: &InstallOwner, target: &PackageVersion) -> Result<(), String> {
        match owner {
            InstallOwner::Npm { .. } => install_with_npm(target, &SystemCommandRunner),
            InstallOwner::Direct { prefix, .. } => {
                install_direct(prefix, target, &SystemCommandRunner)
            }
        }
    }
}

#[derive(Deserialize)]
struct RegistryMetadata {
    #[serde(rename = "dist-tags")]
    dist_tags: DistTags,
    versions: BTreeMap<String, RegistryPackage>,
}

#[derive(Deserialize)]
struct DistTags {
    latest: String,
    test: Option<String>,
}

#[derive(Deserialize)]
struct RegistryPackage {
    #[serde(rename = "aigentHive")]
    aigent_hive: ProductBinding,
}

#[derive(Deserialize)]
struct ProductBinding {
    #[serde(rename = "productVersion")]
    product_version: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DirectReceipt {
    schema_version: u32,
    owner: String,
    product: String,
    version: String,
    package_version: String,
    artifact_sha256: String,
}

#[derive(Deserialize)]
struct NpmManifest {
    name: String,
    version: String,
    #[serde(rename = "aigentHive")]
    aigent_hive: ProductBinding,
}

#[allow(clippy::too_many_lines)]
pub(crate) fn run(arguments: &[String]) -> ExitCode {
    if arguments == ["--help"] {
        print!("{UPDATE_USAGE}");
        return ExitCode::SUCCESS;
    }
    let arguments = match parse_arguments(arguments) {
        Ok(arguments) => arguments,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::from(2);
        }
    };
    let interactive = io::stdin().is_terminal() && io::stderr().is_terminal();
    if !interactive && !arguments.confirm {
        eprintln!(
            "update blocked: `hive update` requires an interactive terminal or explicit --confirm; no update was installed"
        );
        return ExitCode::from(3);
    }
    let user_root = match arguments
        .user_root
        .as_deref()
        .map_or_else(crate::user_install::resolve_user_root_path, |path| {
            Ok(path.to_path_buf())
        }) {
        Ok(root) => root,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::from(2);
        }
    };
    let language = match selected_language(&user_root) {
        Ok(language) => language,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::from(2);
        }
    };
    let current_executable = match env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("error: cannot resolve the running Hive executable: {error}");
            return ExitCode::from(2);
        }
    };
    let executable = arguments
        .handoff_executable
        .as_deref()
        .unwrap_or(&current_executable);
    if cfg!(windows) && arguments.handoff_executable.is_none() && is_direct_owner(executable) {
        if let Err(error) =
            spawn_windows_direct_handoff(&current_executable, &user_root, &arguments)
        {
            eprintln!("error: cannot start Windows direct update handoff: {error}");
            return ExitCode::from(2);
        }
        eprintln!(
            "Windows direct update handoff started; completion continues in the child process."
        );
        return ExitCode::SUCCESS;
    }
    if arguments.handoff_executable.is_some() {
        if let Err(error) = wait_for_windows_direct_unlock(executable) {
            eprintln!("error: {error}");
            return ExitCode::from(2);
        }
    }
    let mut output = io::stderr().lock();
    let outcome = if arguments.confirm {
        let mut input = io::Cursor::new(b"y\n".as_slice());
        update_flow_with_projection_channel(
            &UpdateFlowContext {
                executable,
                user_root: &user_root,
                language,
            },
            &LiveRegistry,
            &LiveInstaller,
            &LiveProjectionRefresher,
            UpdateSelection {
                channel: arguments.channel,
                confirmed: true,
            },
            &mut input,
            &mut output,
        )
    } else {
        let mut input = io::stdin().lock();
        update_flow_with_projection_channel(
            &UpdateFlowContext {
                executable,
                user_root: &user_root,
                language,
            },
            &LiveRegistry,
            &LiveInstaller,
            &LiveProjectionRefresher,
            UpdateSelection {
                channel: arguments.channel,
                confirmed: false,
            },
            &mut input,
            &mut output,
        )
    };
    match outcome {
        Ok(_) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(2)
        }
    }
}

fn parse_arguments(arguments: &[String]) -> Result<UpdateArguments, String> {
    let mut channel = UpdateChannel::Stable;
    let mut channel_seen = false;
    let mut confirm = false;
    let mut handoff_executable = None;
    let mut user_root = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--channel" if !channel_seen => {
                let value = arguments
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --channel".to_owned())?;
                channel = match value.as_str() {
                    "stable" => UpdateChannel::Stable,
                    "test" => UpdateChannel::Test,
                    _ => return Err("--channel must be stable or test".to_owned()),
                };
                channel_seen = true;
                index += 2;
            }
            "--confirm" if !confirm => {
                confirm = true;
                index += 1;
            }
            "--direct-handoff" if handoff_executable.is_none() => {
                let value = arguments
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --direct-handoff".to_owned())?;
                let path = PathBuf::from(value);
                if !path.is_absolute() {
                    return Err("--direct-handoff must be an absolute executable path".to_owned());
                }
                handoff_executable = Some(path);
                index += 2;
            }
            "--user-root" if user_root.is_none() => {
                let value = arguments
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --user-root".to_owned())?;
                let path = PathBuf::from(value);
                if !path.is_absolute() {
                    return Err("--user-root must be an absolute directory".to_owned());
                }
                user_root = Some(path);
                index += 2;
            }
            option => return Err(format!("unknown or duplicate update option: {option}")),
        }
    }
    Ok(UpdateArguments {
        channel,
        confirm,
        handoff_executable,
        user_root,
    })
}

fn is_direct_owner(executable: &Path) -> bool {
    let Ok(expected) = env!("CARGO_PKG_VERSION").parse() else {
        return false;
    };
    matches!(
        discover_owner(executable, expected),
        Ok(InstallOwner::Direct { .. })
    )
}

fn spawn_windows_direct_handoff(
    executable: &Path,
    user_root: &Path,
    arguments: &UpdateArguments,
) -> Result<(), String> {
    if !cfg!(windows) {
        return Err("Windows direct update handoff is unavailable on this platform".to_owned());
    }
    let helper = user_root.join(DIRECT_HANDOFF_HELPER);
    let parent = helper
        .parent()
        .ok_or_else(|| "direct update helper has no parent directory".to_owned())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("cannot create direct update helper directory: {error}"))?;
    if helper.exists() {
        fs::remove_file(&helper)
            .map_err(|error| format!("cannot replace prior direct update helper: {error}"))?;
    }
    fs::copy(executable, &helper)
        .map_err(|error| format!("cannot copy direct update helper: {error}"))?;
    let channel = match arguments.channel {
        UpdateChannel::Stable => "stable",
        UpdateChannel::Test => "test",
    };
    let user_root = user_root
        .to_str()
        .ok_or_else(|| "Hive user root is not UTF-8".to_owned())?;
    let executable = executable
        .to_str()
        .ok_or_else(|| "Hive executable path is not UTF-8".to_owned())?;
    let mut command = Command::new(&helper);
    command.args([
        "update",
        "--direct-handoff",
        executable,
        "--channel",
        channel,
        "--user-root",
        user_root,
    ]);
    if arguments.confirm {
        command.arg("--confirm");
    }
    command
        .spawn()
        .map_err(|error| format!("cannot launch direct update helper: {error}"))?;
    Ok(())
}

fn wait_for_windows_direct_unlock(executable: &Path) -> Result<(), String> {
    if !cfg!(windows) {
        return Ok(());
    }
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match fs::OpenOptions::new().write(true).open(executable) {
            Ok(file) => {
                drop(file);
                return Ok(());
            }
            Err(error)
                if error.kind() == io::ErrorKind::PermissionDenied && Instant::now() < deadline =>
            {
                thread::sleep(Duration::from_millis(50));
            }
            Err(error) => return Err(format!("direct update executable remained locked: {error}")),
        }
    }
}

fn selected_language(user_root: &Path) -> Result<Language, String> {
    let root = crate::user_install::open_user_root_for_setup(user_root)?;
    let config = crate::user_setup::load_operational_config(&root)
        .map_err(|error| error.message().to_owned())?
        .ok_or_else(|| "global Hive setup is required before interactive updates".to_owned())?;
    Ok(match config.interface_language {
        crate::user_setup::InterfaceLanguage::En => Language::En,
        crate::user_setup::InterfaceLanguage::Ko => Language::Ko,
    })
}

#[cfg(test)]
fn update_flow(
    executable: &Path,
    language: Language,
    registry: &impl RegistrySource,
    installer: &impl Installer,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> Result<FlowOutcome, String> {
    update_flow_with_projection(
        &UpdateFlowContext {
            executable,
            user_root: Path::new(""),
            language,
        },
        registry,
        installer,
        &NoopProjectionRefresher,
        input,
        output,
    )
}

#[cfg(test)]
fn update_flow_with_projection(
    context: &UpdateFlowContext<'_>,
    registry: &impl RegistrySource,
    installer: &impl Installer,
    refresher: &impl ProjectionRefresher,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> Result<FlowOutcome, String> {
    update_flow_with_projection_channel(
        context,
        registry,
        installer,
        refresher,
        UpdateSelection {
            channel: UpdateChannel::Stable,
            confirmed: false,
        },
        input,
        output,
    )
}

fn update_flow_with_projection_channel(
    context: &UpdateFlowContext<'_>,
    registry: &impl RegistrySource,
    installer: &impl Installer,
    refresher: &impl ProjectionRefresher,
    selection: UpdateSelection,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> Result<FlowOutcome, String> {
    let current_product: SemVersion = env!("CARGO_PKG_VERSION")
        .parse()
        .map_err(|error| format!("compiled Hive version is invalid: {error}"))?;
    let owner = discover_owner(context.executable, current_product)?;
    let metadata = registry.fetch()?;
    let target = target_package(&metadata, selection.channel)?;
    if target.product.major != current_product.major {
        return Err(format!(
            "the npm distribution targets product {} and requires explicit major-version authority",
            target.product
        ));
    }
    if target <= *owner.package_version() {
        write_current(output, context.language, owner.package_version())?;
        return Ok(FlowOutcome::Current);
    }
    let hosts = refresher.authenticated_hosts(context.user_root)?;
    write_prompt(output, context.language, &owner, &target, &hosts)?;
    if !selection.confirmed {
        let mut answer = String::new();
        let read = input
            .read_line(&mut answer)
            .map_err(|error| format!("cannot read update confirmation: {error}"))?;
        if read == 0 || !accepted(context.language, &answer) {
            write_declined(output, context.language)?;
            return Ok(FlowOutcome::Declined);
        }
    }
    installer.install(&owner, &target)?;
    let activated_owner = discover_owner(context.executable, target.product)?;
    if activated_owner.label() != owner.label() || activated_owner.package_version() != &target {
        return Err("the install owner did not activate the exact requested package".to_owned());
    }
    if !hosts.is_empty() {
        refresher
            .refresh_and_validate(context.executable, context.user_root, &hosts)
            .map_err(|error| {
                format!(
                    "Aigent Hive binary update completed, but the authenticated user projection refresh failed: {error}"
                )
            })?;
    }
    write_installed(output, context.language, &target, &hosts)?;
    Ok(FlowOutcome::Installed)
}

fn target_package(bytes: &[u8], channel: UpdateChannel) -> Result<PackageVersion, String> {
    let metadata: RegistryMetadata = serde_json::from_slice(bytes)
        .map_err(|error| format!("registry response is malformed JSON: {error}"))?;
    let tagged = match channel {
        UpdateChannel::Stable => &metadata.dist_tags.latest,
        UpdateChannel::Test => metadata.dist_tags.test.as_deref().ok_or_else(|| {
            "npm registry does not publish an aigent-hive test channel".to_owned()
        })?,
    };
    let target = parse_package_version(tagged)?;
    let package = metadata.versions.get(&target.exact).ok_or_else(|| {
        "registry latest tag does not name a published package version".to_owned()
    })?;
    let bound_product: SemVersion = package
        .aigent_hive
        .product_version
        .parse()
        .map_err(|error| format!("registry product version is invalid: {error}"))?;
    if bound_product != target.product {
        return Err("registry package version and embedded product version disagree".to_owned());
    }
    Ok(target)
}

fn parse_package_version(value: &str) -> Result<PackageVersion, String> {
    let (product, channel) = if let Some(product) = value.strip_suffix("-test") {
        (product, PackageChannel::Test(None))
    } else if let Some((product, revision)) = value.split_once("-test.") {
        if revision.is_empty()
            || revision.starts_with('0')
            || !revision.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err("npm test package revision must be a positive integer".to_owned());
        }
        let revision = revision
            .parse()
            .map_err(|_| "npm test package revision is out of range".to_owned())?;
        (product, PackageChannel::Test(Some(revision)))
    } else {
        (value, PackageChannel::Stable)
    };
    let product: SemVersion = product
        .parse()
        .map_err(|error| format!("npm package product version is invalid: {error}"))?;
    Ok(PackageVersion {
        product,
        channel,
        exact: value.to_owned(),
    })
}

fn discover_owner(executable: &Path, expected_product: SemVersion) -> Result<InstallOwner, String> {
    let executable = executable
        .canonicalize()
        .map_err(|error| format!("cannot resolve the running Hive executable: {error}"))?;
    ensure_regular(&executable, MAX_BINARY_BYTES)?;
    let expected_executable = if cfg!(windows) { "hive.exe" } else { "hive" };
    if executable.file_name().and_then(|name| name.to_str()) != Some(expected_executable)
        || executable
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            != Some("bin")
    {
        return Err("the running Hive executable is outside a supported install layout".to_owned());
    }
    let prefix = executable
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "the running Hive executable has no installation prefix".to_owned())?;
    let receipt = prefix.join("share/aigent-hive/install-receipt.json");
    match read_optional_regular(&receipt, MAX_MANIFEST_BYTES)? {
        Some(bytes) => discover_direct(&executable, prefix, &bytes, expected_product),
        None => discover_npm(&executable, expected_product),
    }
}

fn discover_direct(
    executable: &Path,
    prefix: &Path,
    bytes: &[u8],
    expected_product: SemVersion,
) -> Result<InstallOwner, String> {
    let receipt: DirectReceipt = serde_json::from_slice(bytes)
        .map_err(|error| format!("direct install receipt is malformed: {error}"))?;
    let version: SemVersion = receipt
        .version
        .parse()
        .map_err(|error| format!("direct install product version is invalid: {error}"))?;
    let package_version = parse_package_version(&receipt.package_version)?;
    if receipt.schema_version != 1
        || receipt.owner != "direct"
        || receipt.product != "aigent-hive"
        || version != expected_product
        || package_version.product != version
    {
        return Err("direct install receipt does not authenticate this Hive version".to_owned());
    }
    let expected_digest = receipt
        .artifact_sha256
        .strip_prefix("sha256:")
        .filter(|digest| digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| "direct install receipt contains an invalid artifact digest".to_owned())?;
    let binary = fs::read(executable)
        .map_err(|error| format!("cannot read the running Hive executable: {error}"))?;
    if sha256_digest(&binary) != format!("sha256:{}", expected_digest.to_ascii_lowercase()) {
        return Err("direct install receipt does not match the running Hive executable".to_owned());
    }
    Ok(InstallOwner::Direct {
        package_version,
        prefix: prefix.to_path_buf(),
    })
}

fn discover_npm(executable: &Path, expected_product: SemVersion) -> Result<InstallOwner, String> {
    let package_root = executable
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "npm native executable has no package root".to_owned())?;
    let manifest_path = package_root.join("package.json");
    let bytes = read_optional_regular(&manifest_path, MAX_MANIFEST_BYTES)?
        .ok_or_else(|| "no authenticated direct or npm install owner was found".to_owned())?;
    let manifest: NpmManifest = serde_json::from_slice(&bytes)
        .map_err(|error| format!("npm platform package manifest is malformed: {error}"))?;
    let expected_name = platform_package_name()?;
    let product: SemVersion = manifest
        .aigent_hive
        .product_version
        .parse()
        .map_err(|error| format!("npm platform product version is invalid: {error}"))?;
    let package_version = parse_package_version(&manifest.version)?;
    if manifest.name != expected_name
        || product != expected_product
        || package_version.product != product
    {
        return Err("npm platform package does not authenticate this Hive executable".to_owned());
    }
    Ok(InstallOwner::Npm { package_version })
}

fn platform_package_name() -> Result<&'static str, String> {
    match (env::consts::OS, env::consts::ARCH) {
        ("macos", "aarch64") => Ok("@aigent-hive/darwin-arm64"),
        ("macos", "x86_64") => Ok("@aigent-hive/darwin-x64"),
        ("linux", "aarch64") => Ok("@aigent-hive/linux-arm64"),
        ("linux", "x86_64") => Ok("@aigent-hive/linux-x64"),
        ("windows", "x86_64") => Ok("@aigent-hive/win32-x64"),
        (operating_system, architecture) => Err(format!(
            "interactive update is unsupported on {operating_system}/{architecture}"
        )),
    }
}

fn ensure_regular(path: &Path, maximum: u64) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        return Err(format!(
            "{} must be a regular non-link file",
            path.display()
        ));
    }
    if metadata.len() > maximum {
        return Err(format!("{} exceeds the size limit", path.display()));
    }
    Ok(())
}

fn read_optional_regular(path: &Path, maximum: u64) -> Result<Option<Vec<u8>>, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            ensure_regular(path, maximum)?;
            fs::read(path)
                .map(Some)
                .map_err(|error| format!("cannot read {}: {error}", path.display()))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("cannot inspect {}: {error}", path.display())),
    }
}

#[cfg(windows)]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
const fn is_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}

fn install_with_npm(target: &PackageVersion, runner: &impl CommandRunner) -> Result<(), String> {
    let program_name = if cfg!(windows) { "npm.cmd" } else { "npm" };
    let program = runner
        .qualify(program_name)
        .map_err(|error| format!("cannot qualify npm install owner: {error}"))?;
    let package = format!("aigent-hive@{}", target.exact);
    let output = runner
        .run(
            &program,
            &["install", "-g", &package],
            INSTALL_TIMEOUT,
            INSTALL_OUTPUT_LIMIT,
        )
        .map_err(|error| format!("npm update failed: {error}"))?;
    if !output.success {
        return Err("npm install owner rejected the exact update".to_owned());
    }
    Ok(())
}

fn install_direct(
    prefix: &Path,
    target: &PackageVersion,
    runner: &impl CommandRunner,
) -> Result<(), String> {
    let extension = if cfg!(windows) { ".ps1" } else { ".sh" };
    let url = format!(
        "https://unpkg.com/aigent-hive@{}/install{}",
        target.exact, extension
    );
    let bytes = crate::update_discovery::fetch_https(&url, MAX_INSTALLER_BYTES)?;
    validate_direct_installer(&bytes, target)?;
    let mut installer = tempfile::Builder::new()
        .prefix("aigent-hive-update-")
        .suffix(extension)
        .tempfile()
        .map_err(|error| format!("cannot stage the direct installer: {error}"))?;
    installer
        .write_all(&bytes)
        .and_then(|()| installer.flush())
        .map_err(|error| format!("cannot write the direct installer: {error}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(installer.path(), fs::Permissions::from_mode(0o700))
            .map_err(|error| format!("cannot protect the staged direct installer: {error}"))?;
    }
    let installer_path_guard = installer.into_temp_path();
    let installer_path = installer_path_guard
        .to_str()
        .ok_or_else(|| "direct installer path is not UTF-8".to_owned())?;
    let prefix = prefix
        .to_str()
        .ok_or_else(|| "direct install prefix is not UTF-8".to_owned())?;
    let output = if cfg!(windows) {
        let program = runner
            .qualify("powershell.exe")
            .map_err(|error| format!("cannot qualify Windows PowerShell 5.1: {error}"))?;
        runner.run(
            &program,
            &[
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                installer_path,
                "-Version",
                &target.product.to_string(),
                "-PackageVersion",
                &target.exact,
                "-Prefix",
                prefix,
            ],
            INSTALL_TIMEOUT,
            INSTALL_OUTPUT_LIMIT,
        )
    } else {
        let program = runner
            .qualify("env")
            .map_err(|error| format!("cannot qualify the direct install environment: {error}"))?;
        let product = format!("AIGENT_HIVE_VERSION={}", target.product);
        let package = format!("AIGENT_HIVE_PACKAGE_VERSION={}", target.exact);
        let prefix = format!("AIGENT_HIVE_PREFIX={prefix}");
        runner.run(
            &program,
            &[&product, &package, &prefix, "sh", installer_path],
            INSTALL_TIMEOUT,
            INSTALL_OUTPUT_LIMIT,
        )
    }
    .map_err(|error| format!("direct update failed: {error}"))?;
    drop(installer_path_guard);
    if !output.success {
        return Err("direct install owner rejected the exact update".to_owned());
    }
    Ok(())
}

fn validate_direct_installer(bytes: &[u8], target: &PackageVersion) -> Result<(), String> {
    let text =
        std::str::from_utf8(bytes).map_err(|_| "direct installer is not valid UTF-8".to_owned())?;
    if text.lines().any(|line| line.contains("= \"__AIGENT_HIVE_")) {
        return Err("direct installer contains unresolved release markers".to_owned());
    }
    let expected_product = if cfg!(windows) {
        format!("[string]$Version = \"{}\"", target.product)
    } else {
        format!("embedded_product_version='{}'", target.product)
    };
    let expected_package = if cfg!(windows) {
        format!("[string]$PackageVersion = \"{}\"", target.exact)
    } else {
        format!("embedded_package_version='{}'", target.exact)
    };
    if !text.contains(&expected_product) || !text.contains(&expected_package) {
        return Err("direct installer does not bind the exact requested release".to_owned());
    }
    Ok(())
}

fn accepted(language: Language, answer: &str) -> bool {
    let answer = answer.trim();
    match language {
        Language::En => answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes"),
        Language::Ko => {
            answer.eq_ignore_ascii_case("y")
                || answer.eq_ignore_ascii_case("yes")
                || matches!(answer, "예" | "네")
        }
    }
}

fn write_prompt(
    output: &mut impl Write,
    language: Language,
    owner: &InstallOwner,
    target: &PackageVersion,
    hosts: &[String],
) -> Result<(), String> {
    let command = match owner {
        InstallOwner::Npm { .. } => format!("npm install -g aigent-hive@{}", target.exact),
        InstallOwner::Direct { .. } if cfg!(windows) => format!(
            "download https://unpkg.com/aigent-hive@{}/install.ps1; powershell.exe -File install.ps1 -Version {} -PackageVersion {}",
            target.exact, target.product, target.exact
        ),
        InstallOwner::Direct { .. } => format!(
            "download https://unpkg.com/aigent-hive@{}/install.sh; sh install.sh with AIGENT_HIVE_VERSION={} AIGENT_HIVE_PACKAGE_VERSION={}",
            target.exact, target.product, target.exact
        ),
    };
    match language {
        Language::En => write!(
            output,
            "Aigent Hive update available: {} -> {}\nInstall owner: {}\nExact operation: {}\nUser projection refresh: {}\nProceed? [y/N]: ",
            owner.package_version().exact,
            target.exact,
            owner.label(),
            command,
            projection_scope_message(Language::En, hosts),
        ),
        Language::Ko => write!(
            output,
            "Aigent Hive 갱신 가능: {} -> {}\n설치 소유자: {}\n정확한 작업: {}\n사용자 투영 갱신: {}\n진행할까요? [y/N]: ",
            owner.package_version().exact,
            target.exact,
            owner.label(),
            command,
            projection_scope_message(Language::Ko, hosts),
        ),
    }
    .and_then(|()| output.flush())
    .map_err(|error| format!("cannot display the update confirmation: {error}"))
}

fn write_current(
    output: &mut impl Write,
    language: Language,
    current: &PackageVersion,
) -> Result<(), String> {
    let message = match language {
        Language::En => format!("Aigent Hive is current at {}.\n", current.exact),
        Language::Ko => format!("Aigent Hive 최신 버전 사용 중: {}.\n", current.exact),
    };
    output
        .write_all(message.as_bytes())
        .map_err(|error| format!("cannot display update status: {error}"))
}

fn write_declined(output: &mut impl Write, language: Language) -> Result<(), String> {
    let message = match language {
        Language::En => "Update declined; no files were changed.\n",
        Language::Ko => "갱신 거절, 파일 변경 없음.\n",
    };
    output
        .write_all(message.as_bytes())
        .map_err(|error| format!("cannot display update status: {error}"))
}

fn write_installed(
    output: &mut impl Write,
    language: Language,
    target: &PackageVersion,
    hosts: &[String],
) -> Result<(), String> {
    let message = match language {
        Language::En if hosts.is_empty() => format!(
            "Aigent Hive update complete: {}. No authenticated user projection was present; no user files were changed.\n",
            target.exact
        ),
        Language::Ko if hosts.is_empty() => format!(
            "Aigent Hive 갱신 완료: {}. 인증된 사용자 투영이 없어 사용자 파일은 변경하지 않았습니다.\n",
            target.exact
        ),
        Language::En => format!(
            "Aigent Hive update complete: {}. Refreshed and validated user projection hosts: {}.\n",
            target.exact,
            hosts.join(", ")
        ),
        Language::Ko => format!(
            "Aigent Hive 갱신 완료: {}. 갱신·검증한 사용자 투영 호스트: {}.\n",
            target.exact,
            hosts.join(", ")
        ),
    };
    output
        .write_all(message.as_bytes())
        .map_err(|error| format!("cannot display update status: {error}"))
}

fn projection_scope_message(language: Language, hosts: &[String]) -> String {
    if hosts.is_empty() {
        return match language {
            Language::En => "none (binary-only update)".to_owned(),
            Language::Ko => "없음(바이너리만 갱신)".to_owned(),
        };
    }
    hosts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::io::Cursor;
    use tempfile::tempdir;

    struct FakeRegistry(Vec<u8>);

    impl RegistrySource for FakeRegistry {
        fn fetch(&self) -> Result<Vec<u8>, String> {
            Ok(self.0.clone())
        }
    }

    struct FakeInstaller {
        calls: RefCell<Vec<String>>,
        manifest: Option<PathBuf>,
    }

    impl Default for FakeInstaller {
        fn default() -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
                manifest: None,
            }
        }
    }

    impl Installer for FakeInstaller {
        fn install(&self, _owner: &InstallOwner, target: &PackageVersion) -> Result<(), String> {
            self.calls.borrow_mut().push(target.exact.clone());
            if let Some(manifest) = &self.manifest {
                fs::write(
                    manifest,
                    format!(
                        r#"{{"name":"{}","version":"{}","aigentHive":{{"productVersion":"{}"}}}}"#,
                        platform_package_name().expect("supported test platform"),
                        target.exact,
                        target.product
                    ),
                )
                .map_err(|error| error.to_string())?;
            }
            Ok(())
        }
    }

    struct FakeProjectionRefresher {
        hosts: Result<Vec<String>, String>,
        refresh: Result<(), String>,
        calls: RefCell<Vec<(PathBuf, PathBuf, Vec<String>)>>,
    }

    impl FakeProjectionRefresher {
        fn ready(hosts: &[&str]) -> Self {
            Self {
                hosts: Ok(hosts.iter().map(|host| (*host).to_owned()).collect()),
                refresh: Ok(()),
                calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl ProjectionRefresher for FakeProjectionRefresher {
        fn authenticated_hosts(&self, _user_root: &Path) -> Result<Vec<String>, String> {
            self.hosts.clone()
        }

        fn refresh_and_validate(
            &self,
            executable: &Path,
            user_root: &Path,
            hosts: &[String],
        ) -> Result<(), String> {
            self.calls.borrow_mut().push((
                executable.to_path_buf(),
                user_root.to_path_buf(),
                hosts.to_vec(),
            ));
            self.refresh.clone()
        }
    }

    fn metadata(version: &str) -> FakeRegistry {
        metadata_with_channels(version, None)
    }

    fn metadata_with_channels(stable: &str, test: Option<&str>) -> FakeRegistry {
        let versions = [Some(stable), test]
            .into_iter()
            .flatten()
            .map(|version| {
                let product = version
                    .strip_suffix("-test")
                    .or_else(|| version.split_once("-test.").map(|(product, _)| product))
                    .unwrap_or(version);
                format!(r#""{version}":{{"aigentHive":{{"productVersion":"{product}"}}}}"#)
            })
            .collect::<Vec<_>>()
            .join(",");
        let test_tag = test.map_or_else(String::new, |version| format!(r#","test":"{version}""#));
        FakeRegistry(
            format!(
                r#"{{"dist-tags":{{"latest":"{stable}"{test_tag}}},"versions":{{{versions}}}}}"#
            )
            .into_bytes(),
        )
    }

    fn package(revision: u64) -> String {
        format!("{}-test.{revision}", env!("CARGO_PKG_VERSION"))
    }

    fn stable_package() -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }

    fn fake_npm_install(package_version: &str) -> (tempfile::TempDir, PathBuf) {
        let root = tempdir().expect("package root");
        let binary = root
            .path()
            .join("bin")
            .join(if cfg!(windows) { "hive.exe" } else { "hive" });
        fs::create_dir_all(binary.parent().expect("bin parent")).expect("bin");
        fs::write(&binary, b"fake hive binary").expect("binary");
        fs::write(
            root.path().join("package.json"),
            format!(
                r#"{{"name":"{}","version":"{}","aigentHive":{{"productVersion":"{}"}}}}"#,
                platform_package_name().expect("supported test platform"),
                package_version,
                env!("CARGO_PKG_VERSION")
            ),
        )
        .expect("manifest");
        (root, binary)
    }

    #[test]
    fn package_versions_order_tests_before_stable_and_then_next_product() {
        let bare = parse_package_version("0.8.0-test").expect("bare test");
        let first = parse_package_version("0.8.0-test.1").expect("first");
        let second = parse_package_version("0.8.0-test.2").expect("second");
        let stable = parse_package_version("0.8.0").expect("stable");
        let next = parse_package_version("0.8.1-test.1").expect("next");
        assert!(bare < first);
        assert!(first < second);
        assert!(second < stable);
        assert!(stable < next);
        assert!(second < next);
        assert!(parse_package_version("0.8.0-test.0").is_err());
        assert!(parse_package_version("0.8.0-test.01").is_err());
        assert!(parse_package_version("0.8.0-preview.1").is_err());
    }

    #[test]
    fn npm_owner_accepts_the_default_bare_test_package_binding() {
        let current = format!("{}-test", env!("CARGO_PKG_VERSION"));
        let (_root, binary) = fake_npm_install(&current);
        let product: SemVersion = env!("CARGO_PKG_VERSION").parse().expect("product");
        let owner = discover_owner(&binary, product).expect("npm owner");
        assert_eq!(owner.label(), "npm");
        assert_eq!(owner.package_version().exact, current);
    }

    #[test]
    fn npm_owner_accepts_legacy_test_package_binding() {
        let current = package(1);
        let (_root, binary) = fake_npm_install(&current);
        let product: SemVersion = env!("CARGO_PKG_VERSION").parse().expect("product");
        let owner = discover_owner(&binary, product).expect("npm owner");
        assert_eq!(owner.label(), "npm");
        assert_eq!(owner.package_version().exact, current);
    }

    #[test]
    fn declined_update_does_not_invoke_the_install_owner() {
        let current = package(1);
        let target = stable_package();
        let (_root, binary) = fake_npm_install(&current);
        let installer = FakeInstaller::default();
        let mut input = Cursor::new(b"n\n");
        let mut output = Vec::new();
        let outcome = update_flow(
            &binary,
            Language::En,
            &metadata(&target),
            &installer,
            &mut input,
            &mut output,
        )
        .expect("declined");
        assert_eq!(outcome, FlowOutcome::Declined);
        assert!(installer.calls.borrow().is_empty());
        assert!(String::from_utf8(output)
            .expect("output")
            .contains("no files were changed"));
    }

    #[test]
    fn current_package_skips_confirmation_and_installation() {
        let current = stable_package();
        let (_root, binary) = fake_npm_install(&current);
        let installer = FakeInstaller::default();
        let mut input = Cursor::new(Vec::<u8>::new());
        let mut output = Vec::new();
        let outcome = update_flow(
            &binary,
            Language::Ko,
            &metadata(&current),
            &installer,
            &mut input,
            &mut output,
        )
        .expect("current");
        assert_eq!(outcome, FlowOutcome::Current);
        assert!(installer.calls.borrow().is_empty());
        assert!(String::from_utf8(output)
            .expect("output")
            .contains("최신 버전 사용 중"));
    }

    #[test]
    fn accepted_update_invokes_owner_once_and_revalidates_exact_package() {
        let current = package(1);
        let target = stable_package();
        let (root, binary) = fake_npm_install(&current);
        let installer = FakeInstaller {
            calls: RefCell::new(Vec::new()),
            manifest: Some(root.path().join("package.json")),
        };
        let mut input = Cursor::new("예\n".as_bytes());
        let mut output = Vec::new();
        let outcome = update_flow(
            &binary,
            Language::Ko,
            &metadata(&target),
            &installer,
            &mut input,
            &mut output,
        )
        .expect("installed");
        assert_eq!(outcome, FlowOutcome::Installed);
        assert_eq!(installer.calls.borrow().as_slice(), [target]);
        assert!(String::from_utf8(output)
            .expect("output")
            .contains("갱신 완료"));
    }

    #[test]
    fn explicit_test_channel_selects_only_the_test_tag() {
        let current = package(1);
        let target = package(2);
        let stable = stable_package();
        let (root, binary) = fake_npm_install(&current);
        let installer = FakeInstaller {
            calls: RefCell::new(Vec::new()),
            manifest: Some(root.path().join("package.json")),
        };
        let mut input = Cursor::new(b"y\n");
        let mut output = Vec::new();

        let outcome = update_flow_with_projection_channel(
            &UpdateFlowContext {
                executable: &binary,
                user_root: Path::new(""),
                language: Language::En,
            },
            &metadata_with_channels(&stable, Some(&target)),
            &installer,
            &NoopProjectionRefresher,
            UpdateSelection {
                channel: UpdateChannel::Test,
                confirmed: false,
            },
            &mut input,
            &mut output,
        )
        .expect("test update installed");

        assert_eq!(outcome, FlowOutcome::Installed);
        assert_eq!(installer.calls.borrow().as_slice(), [target]);
        assert!(String::from_utf8(output)
            .expect("output")
            .contains(&format!("-> {}", package(2))));
    }

    #[test]
    fn explicit_test_channel_requires_a_published_test_tag() {
        let current = package(1);
        let (_root, binary) = fake_npm_install(&current);
        let installer = FakeInstaller::default();
        let mut input = Cursor::new(Vec::<u8>::new());
        let mut output = Vec::new();

        let error = update_flow_with_projection_channel(
            &UpdateFlowContext {
                executable: &binary,
                user_root: Path::new(""),
                language: Language::En,
            },
            &metadata(&stable_package()),
            &installer,
            &NoopProjectionRefresher,
            UpdateSelection {
                channel: UpdateChannel::Test,
                confirmed: false,
            },
            &mut input,
            &mut output,
        )
        .expect_err("missing test channel");

        assert_eq!(
            error,
            "npm registry does not publish an aigent-hive test channel"
        );
        assert!(installer.calls.borrow().is_empty());
    }

    #[test]
    fn update_arguments_require_explicit_known_values() {
        assert_eq!(
            parse_arguments(&[
                "--channel".to_owned(),
                "test".to_owned(),
                "--confirm".to_owned()
            ])
            .expect("test arguments")
            .channel,
            UpdateChannel::Test
        );
        assert!(parse_arguments(&["--channel".to_owned(), "preview".to_owned()]).is_err());
        assert!(parse_arguments(&["--user-root".to_owned(), "relative".to_owned()]).is_err());
        assert!(parse_arguments(&["--confirm".to_owned(), "--confirm".to_owned()]).is_err());
    }

    #[test]
    fn projection_refresh_requires_the_action_for_each_install_mode() {
        assert_eq!(projection_refresh_action("--apply"), "InstallHiveUser");
        assert_eq!(projection_refresh_action("--validate"), "ValidateHiveUser");
        assert!(projection_refresh_reported_success(
            projection_refresh_action("--apply"),
            &ChildActionResult {
                action: "InstallHiveUser".to_owned(),
                status: "success".to_owned(),
                exit_code: 0,
            },
        ));
        assert!(projection_refresh_reported_success(
            projection_refresh_action("--validate"),
            &ChildActionResult {
                action: "ValidateHiveUser".to_owned(),
                status: "success".to_owned(),
                exit_code: 0,
            },
        ));
        assert!(!projection_refresh_reported_success(
            projection_refresh_action("--validate"),
            &ChildActionResult {
                action: "InstallHiveUser".to_owned(),
                status: "success".to_owned(),
                exit_code: 0,
            },
        ));
    }

    #[cfg(windows)]
    #[test]
    fn direct_installer_allows_the_optional_signer_fallback_but_not_bound_markers() {
        let product = env!("CARGO_PKG_VERSION");
        let package = format!("{product}-test.2");
        let installer = format!(
            "[string]$Version = \"{product}\"\n[string]$PackageVersion = \"{package}\"\n$ExpectedArchiveSha256 = \"abc\"\nif ($AuthorizedSignerThumbprint -like \"__AIGENT_HIVE_*\") {{ }}\n"
        );
        let target = parse_package_version(&package).expect("test package");
        validate_direct_installer(installer.as_bytes(), &target).expect("optional signer fallback");
        let unresolved = installer.replace(
            "$ExpectedArchiveSha256 = \"abc\"",
            "$ExpectedArchiveSha256 = \"__AIGENT_HIVE_ARCHIVE__\"",
        );
        assert!(validate_direct_installer(unresolved.as_bytes(), &target).is_err());
    }

    #[test]
    fn accepted_update_refreshes_and_validates_only_the_authenticated_saved_hosts() {
        let current = package(1);
        let target = stable_package();
        let (root, binary) = fake_npm_install(&current);
        let installer = FakeInstaller {
            calls: RefCell::new(Vec::new()),
            manifest: Some(root.path().join("package.json")),
        };
        let user_root = tempdir().expect("user root");
        let refresher = FakeProjectionRefresher::ready(&["claude", "codex"]);
        let mut input = Cursor::new(b"y\n");
        let mut output = Vec::new();

        let outcome = update_flow_with_projection(
            &UpdateFlowContext {
                executable: &binary,
                user_root: user_root.path(),
                language: Language::En,
            },
            &metadata(&target),
            &installer,
            &refresher,
            &mut input,
            &mut output,
        )
        .expect("installed and refreshed");

        assert_eq!(outcome, FlowOutcome::Installed);
        assert_eq!(installer.calls.borrow().as_slice(), [target]);
        assert_eq!(
            refresher.calls.borrow().as_slice(),
            [(
                binary.clone(),
                user_root.path().to_path_buf(),
                vec!["claude".to_owned(), "codex".to_owned()]
            )]
        );
        let text = String::from_utf8(output).expect("output");
        assert!(text.contains("User projection refresh: claude, codex"));
        assert!(text.contains("Refreshed and validated user projection hosts: claude, codex"));
    }

    #[test]
    fn absent_saved_projection_scope_keeps_the_update_binary_only() {
        let current = package(1);
        let target = stable_package();
        let (root, binary) = fake_npm_install(&current);
        let installer = FakeInstaller {
            calls: RefCell::new(Vec::new()),
            manifest: Some(root.path().join("package.json")),
        };
        let user_root = tempdir().expect("user root");
        let refresher = FakeProjectionRefresher::ready(&[]);
        let mut input = Cursor::new(b"y\n");
        let mut output = Vec::new();

        update_flow_with_projection(
            &UpdateFlowContext {
                executable: &binary,
                user_root: user_root.path(),
                language: Language::En,
            },
            &metadata(&target),
            &installer,
            &refresher,
            &mut input,
            &mut output,
        )
        .expect("binary-only update");

        assert!(refresher.calls.borrow().is_empty());
        assert!(String::from_utf8(output)
            .expect("output")
            .contains("No authenticated user projection was present"));
    }

    #[test]
    fn invalid_saved_projection_scope_blocks_the_binary_update_before_confirmation() {
        let current = package(1);
        let target = stable_package();
        let (_root, binary) = fake_npm_install(&current);
        let installer = FakeInstaller::default();
        let user_root = tempdir().expect("user root");
        let refresher = FakeProjectionRefresher {
            hosts: Err(
                "installed ownership manifest is malformed: .hive/install/codex.json".to_owned(),
            ),
            refresh: Ok(()),
            calls: RefCell::new(Vec::new()),
        };
        let mut input = Cursor::new(b"y\n");
        let mut output = Vec::new();

        let error = update_flow_with_projection(
            &UpdateFlowContext {
                executable: &binary,
                user_root: user_root.path(),
                language: Language::En,
            },
            &metadata(&target),
            &installer,
            &refresher,
            &mut input,
            &mut output,
        )
        .expect_err("invalid scope blocks update");

        assert!(error.contains("ownership manifest is malformed"));
        assert!(installer.calls.borrow().is_empty());
        assert!(refresher.calls.borrow().is_empty());
    }

    #[test]
    fn post_install_projection_refresh_failure_is_reported_without_claiming_success() {
        let current = package(1);
        let target = stable_package();
        let (root, binary) = fake_npm_install(&current);
        let installer = FakeInstaller {
            calls: RefCell::new(Vec::new()),
            manifest: Some(root.path().join("package.json")),
        };
        let user_root = tempdir().expect("user root");
        let refresher = FakeProjectionRefresher {
            hosts: Ok(vec!["codex".to_owned()]),
            refresh: Err(
                "activated Hive user projection --validate command did not report success"
                    .to_owned(),
            ),
            calls: RefCell::new(Vec::new()),
        };
        let mut input = Cursor::new(b"y\n");
        let mut output = Vec::new();

        let error = update_flow_with_projection(
            &UpdateFlowContext {
                executable: &binary,
                user_root: user_root.path(),
                language: Language::En,
            },
            &metadata(&target),
            &installer,
            &refresher,
            &mut input,
            &mut output,
        )
        .expect_err("refresh failure");

        assert!(error.contains("binary update completed"));
        assert_eq!(installer.calls.borrow().as_slice(), [target]);
        assert_eq!(refresher.calls.borrow().len(), 1);
        assert!(!String::from_utf8(output)
            .expect("prompt output")
            .contains("update complete"));
    }

    #[test]
    fn direct_owner_requires_package_version_and_matching_binary_digest() {
        let root = tempdir().expect("direct prefix");
        let binary = root
            .path()
            .join("bin")
            .join(if cfg!(windows) { "hive.exe" } else { "hive" });
        fs::create_dir_all(binary.parent().expect("bin parent")).expect("bin");
        fs::write(&binary, b"direct hive binary").expect("binary");
        let receipt = root.path().join("share/aigent-hive/install-receipt.json");
        fs::create_dir_all(receipt.parent().expect("receipt parent")).expect("share");
        fs::write(
            &receipt,
            format!(
                r#"{{"schema_version":1,"owner":"direct","product":"aigent-hive","version":"{}","package_version":"{}-test.1","artifact_sha256":"{}"}}"#,
                env!("CARGO_PKG_VERSION"),
                env!("CARGO_PKG_VERSION"),
                sha256_digest(b"direct hive binary")
            ),
        )
        .expect("receipt");
        let product: SemVersion = env!("CARGO_PKG_VERSION").parse().expect("product");
        let owner = discover_owner(&binary, product).expect("direct owner");
        assert_eq!(owner.label(), "direct");
        assert_eq!(
            owner.package_version().exact,
            format!("{}-test.1", env!("CARGO_PKG_VERSION"))
        );
    }
}
