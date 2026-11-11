use serde_json::{json, Value};
use std::{fs, path::Path, process::Command};

fn prune(root: &Path, mode: &str) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_hive"))
        .args(["prune", "--user-root"])
        .arg(root)
        .args([mode, "--output", "json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"]["report"].clone()
}

#[test]
fn prune_confirm_removes_large_unused_executables_and_preserves_recorded_files() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().join("사용자 루트 with spaces");
    let monitor = root.join(".hive/runtime/usage-monitor");
    let bin = monitor.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let bytes = vec![0x5a; 2 * 1024 * 1024 + 17];
    for name in [
        "hive-current.exe",
        "hive-previous.exe",
        "hive-old-a.exe",
        "hive-old-b.exe",
    ] {
        fs::write(bin.join(name), &bytes).unwrap();
    }
    fs::write(bin.join("foreign.txt"), b"foreign bytes").unwrap();
    let config = serde_json::to_vec(&json!({
        "schema_version":1,"enabled":false,"version":"0.14.2",
        "executable":bin.join("hive-current.exe"),
        "executable_digest":hive_core::sha256_digest(&bytes),
        "task_name":"fixture-only","arguments":"fixture-only"
    }))
    .unwrap();
    let journal = serde_json::to_vec(&json!({
        "from":{"executable":bin.join("hive-previous.exe")},
        "to":{"executable":bin.join("hive-current.exe")}
    }))
    .unwrap();
    fs::write(monitor.join("config.json"), &config).unwrap();
    fs::write(monitor.join("upgrade.json"), &journal).unwrap();

    let preview = prune(&root, "--dry-run");
    assert_eq!(preview["candidates"].as_array().unwrap().len(), 2);
    for name in [
        "hive-current.exe",
        "hive-previous.exe",
        "hive-old-a.exe",
        "hive-old-b.exe",
    ] {
        assert_eq!(fs::read(bin.join(name)).unwrap(), bytes);
    }
    let removed = prune(&root, "--confirm");
    assert_eq!(removed["removed"].as_array().unwrap().len(), 2);
    assert!(removed["skipped"].as_array().unwrap().is_empty());
    assert!(!bin.join("hive-old-a.exe").exists());
    assert!(!bin.join("hive-old-b.exe").exists());
    for name in ["hive-current.exe", "hive-previous.exe"] {
        assert_eq!(fs::read(bin.join(name)).unwrap(), bytes);
    }
    assert_eq!(fs::read(bin.join("foreign.txt")).unwrap(), b"foreign bytes");
    assert_eq!(fs::read(monitor.join("config.json")).unwrap(), config);
    assert_eq!(fs::read(monitor.join("upgrade.json")).unwrap(), journal);
    assert!(prune(&root, "--dry-run")["candidates"]
        .as_array()
        .unwrap()
        .is_empty());
}
