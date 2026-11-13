//! Exercise the actual build parser and its compiled inclusion boundary.
#[allow(dead_code)]
#[path = "../build.rs"]
mod build_script;

#[allow(dead_code)]
mod embedded {
    include!(concat!(env!("OUT_DIR"), "/historical_user_plugins.rs"));
}

#[test]
fn compiled_historical_tables_match_every_registered_plugin_byte() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let registry: serde_yaml::Value =
        serde_yaml::from_str(include_str!("../../../harness/project-bases/registry.yml")).unwrap();
    let expected = registry["releases"]
        .as_sequence()
        .unwrap()
        .iter()
        .filter_map(|entry| {
            let version = entry["version"].as_str().unwrap();
            (entry["user_projection_digest"].is_string() && version != "0.8.0").then_some(version)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        embedded::HISTORICAL_USER_PLUGIN_RELEASES
            .iter()
            .map(|(version, _)| *version)
            .collect::<Vec<_>>(),
        expected
    );
    for (version, files) in embedded::HISTORICAL_USER_PLUGIN_RELEASES {
        for (relative, digest) in *files {
            let bytes = std::fs::read(root.join(format!(
                "harness/user-bases/{version}/plugins/aigent-hive/{relative}"
            )))
            .unwrap();
            assert_eq!(&hive_core::sha256_digest(&bytes), digest);
        }
    }
    for (version, files) in embedded::HISTORICAL_USER_SKILL_CONTENTS {
        for (relative, bytes) in *files {
            assert_eq!(
                *bytes,
                std::fs::read(root.join(format!(
                    "harness/user-bases/{version}/plugins/aigent-hive/{relative}"
                )))
                .unwrap()
            );
        }
    }
}
