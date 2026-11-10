//! Synthetic public-API interleaving; not an assertion about a particular CLI race.
use hive_wiki::{shared, store::RagStore};
use std::{env, fs, path::PathBuf};

fn main() {
    let root = PathBuf::from(env::args_os().nth(1).expect("new synthetic work directory"));
    assert!(!root.exists(), "never reuse an existing directory");
    assert!(root.is_absolute());
    let source = env::current_dir().unwrap();
    assert!(root.starts_with(source.join("tests/work")));
    assert!(!source.join("tests/work").is_symlink());
    let user = root.join("user");
    let a = root.join("project-a");
    let b = root.join("project-b");
    for path in [&user, &a, &b] {
        fs::create_dir_all(path).unwrap();
    }
    let store = RagStore::open(&user).unwrap();
    store.ensure_registry().unwrap();
    let project = |id: &str, path: PathBuf| shared::RegisteredProject {
        id: id.into(), root: path, enabled: true,
        language: shared::KnowledgeLanguage::Ko,
        visibility: shared::KnowledgeVisibility::ProjectPrivate,
    };
    store.register_project_atomic(project("synthetic-a", a)).unwrap();
    // Reader 1 loads an older projects.yml before writer 2 adds B.
    let stale = shared::load_project_registry(&user).unwrap();
    store.register_project_atomic(project("synthetic-b", b)).unwrap();
    // Both store calls take their normal locks, but this input predates registration B.
    store.sync_project_registry(&stale).unwrap();
    let projects = shared::load_project_registry(&user).unwrap();
    let registry = store.load_registry().unwrap();
    let record = registry.collections.iter().find(|c| c.source_project_id.as_deref() == Some("synthetic-b")).unwrap();
    println!("projects_rows={} enabled_b={} collection_b_state={:?}", projects.projects.len(),
             projects.projects.iter().any(|p| p.id == "synthetic-b" && p.enabled), record.state);
    assert_eq!(projects.projects.len(), 2);
    assert_eq!(record.state, hive_wiki::collection::CollectionState::Detached);
}
