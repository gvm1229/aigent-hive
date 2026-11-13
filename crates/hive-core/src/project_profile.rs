//! Read-only project facts from bounded local marker files. No commands are executed.
use cap_std::{ambient_authority, fs::Dir};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::Read, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Fact {
    pub name: String,
    pub confidence: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct ProjectProfile {
    pub languages: Vec<Fact>,
    pub frameworks: Vec<Fact>,
    pub package_managers: Vec<Fact>,
    pub test_tools: Vec<Fact>,
    pub commands: BTreeMap<String, Fact>,
    pub structure: Vec<Fact>,
    pub category: String,
}

fn add(facts: &mut Vec<Fact>, name: &str, path: &str) {
    if let Some(fact) = facts.iter_mut().find(|fact| fact.name == name) {
        if !fact.evidence.iter().any(|item| item == path) {
            fact.evidence.push(path.to_owned());
        }
    } else {
        facts.push(Fact {
            name: name.into(),
            confidence: "high".into(),
            evidence: vec![path.into()],
        });
    }
}

fn text(dir: &Dir, path: &str) -> Option<String> {
    // Capability-relative opens reject symlink escapes; bound even concurrently growing files.
    let file = dir.open(path).ok()?;
    if !file.metadata().ok()?.is_file() {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(1_048_577).read_to_end(&mut bytes).ok()?;
    if bytes.len() > 1_048_576 {
        return None;
    }
    String::from_utf8(bytes).ok()
}

impl ProjectProfile {
    #[must_use]
    pub fn project_kind(&self) -> &str {
        if self.category == "docs" {
            "docs"
        } else if self.category == "general" || self.category.is_empty() {
            "general"
        } else {
            "coding"
        }
    }

    /// Infer facts without running tools, reading nested source, or modifying the target.
    #[must_use]
    pub fn detect(target: &Path) -> Self {
        let Ok(dir) = Dir::open_ambient_dir(target, ambient_authority()) else {
            return Self {
                category: "general".into(),
                ..Self::default()
            };
        };
        Self::detect_in(&dir)
    }

    /// Read facts through the caller's pinned project capability.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn detect_in(dir: &Dir) -> Self {
        let mut result = Self {
            category: "general".into(),
            ..Self::default()
        };
        let mut categories = Vec::new();
        if let Some(package) = text(dir, "package.json")
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .filter(serde_json::Value::is_object)
        {
            add(&mut result.languages, "JavaScript", "package.json");
            categories.push(if package.get("bin").is_some() {
                "cli"
            } else {
                "web"
            });
            if package.get("workspaces").is_some() {
                add(&mut result.structure, "monorepo", "package.json");
            }
            for field in ["dependencies", "devDependencies"] {
                if let Some(deps) = package[field].as_object() {
                    for (dependency, language) in [("typescript", "TypeScript")] {
                        if deps.contains_key(dependency) {
                            add(&mut result.languages, language, "package.json");
                        }
                    }
                    for (dependency, name) in [
                        ("next", "Next.js"),
                        ("vite", "Vite"),
                        ("react", "React"),
                        ("vue", "Vue"),
                        ("svelte", "Svelte"),
                        ("astro", "Astro"),
                        ("@angular/core", "Angular"),
                        ("express", "Express"),
                    ] {
                        if deps.contains_key(dependency) {
                            add(&mut result.frameworks, name, "package.json");
                        }
                    }
                    for name in ["vitest", "jest", "playwright", "@playwright/test", "mocha"] {
                        if deps.contains_key(name) {
                            add(&mut result.test_tools, name, "package.json");
                        }
                    }
                    if deps.contains_key("express") {
                        categories.push("server");
                    }
                }
            }
            if let Some(manager) = package["packageManager"]
                .as_str()
                .and_then(|s| s.split('@').next())
                .filter(|s| matches!(*s, "npm" | "pnpm" | "yarn" | "bun"))
            {
                add(&mut result.package_managers, manager, "package.json");
            }
            for (file, manager) in [
                ("pnpm-lock.yaml", "pnpm"),
                ("yarn.lock", "yarn"),
                ("package-lock.json", "npm"),
                ("bun.lock", "bun"),
                ("bun.lockb", "bun"),
            ] {
                if dir.metadata(file).is_ok_and(|m| m.is_file()) {
                    add(&mut result.package_managers, manager, file);
                }
            }
            if result.package_managers.is_empty() {
                result.package_managers.push(Fact {
                    name: "npm".into(),
                    confidence: "low".into(),
                    evidence: vec!["package.json".into()],
                });
            }
            let manager = &result.package_managers[0].name;
            if let Some(scripts) = package["scripts"].as_object() {
                for name in ["build", "test", "lint", "dev", "check"] {
                    if scripts
                        .get(name)
                        .and_then(serde_json::Value::as_str)
                        .is_some()
                    {
                        result.commands.insert(
                            name.into(),
                            Fact {
                                name: format!("{manager} run {name}"),
                                confidence: "high".into(),
                                evidence: vec!["package.json".into()],
                            },
                        );
                    }
                }
            }
        }
        for (file, framework) in [
            ("next.config.ts", "Next.js"),
            ("next.config.js", "Next.js"),
            ("next.config.mjs", "Next.js"),
            ("vite.config.ts", "Vite"),
            ("vite.config.js", "Vite"),
            ("svelte.config.js", "Svelte"),
            ("astro.config.mjs", "Astro"),
            ("angular.json", "Angular"),
        ] {
            if text(dir, file).is_some() {
                add(&mut result.frameworks, framework, file);
                categories.push("web");
            }
        }
        if text(dir, "tsconfig.json").is_some() {
            add(&mut result.languages, "TypeScript", "tsconfig.json");
            categories.push("web");
        }
        if let Some(cargo) =
            text(dir, "Cargo.toml").and_then(|s| toml::from_str::<toml::Value>(&s).ok())
        {
            add(&mut result.languages, "Rust", "Cargo.toml");
            add(&mut result.package_managers, "cargo", "Cargo.toml");
            categories.push("rust");
            if cargo.get("workspace").is_some() {
                add(&mut result.structure, "monorepo", "Cargo.toml");
            }
            for (key, command) in [
                ("build", "cargo build"),
                ("test", "cargo test"),
                ("lint", "cargo clippy"),
            ] {
                result.commands.insert(
                    key.into(),
                    Fact {
                        name: command.into(),
                        confidence: "high".into(),
                        evidence: vec!["Cargo.toml".into()],
                    },
                );
            }
            add(&mut result.test_tools, "cargo test", "Cargo.toml");
        }
        if let Some(py) =
            text(dir, "pyproject.toml").and_then(|s| toml::from_str::<toml::Value>(&s).ok())
        {
            add(&mut result.languages, "Python", "pyproject.toml");
            categories.push("python");
            if py.get("tool").and_then(|t| t.get("poetry")).is_some() {
                add(&mut result.package_managers, "poetry", "pyproject.toml");
            }
            for tool in ["pytest", "ruff"] {
                if py.get("tool").and_then(|t| t.get(tool)).is_some() {
                    add(&mut result.test_tools, tool, "pyproject.toml");
                    let key = if tool == "pytest" { "test" } else { "lint" };
                    result.commands.insert(
                        key.into(),
                        Fact {
                            name: if tool == "ruff" {
                                "ruff check .".into()
                            } else {
                                "pytest".into()
                            },
                            confidence: "high".into(),
                            evidence: vec!["pyproject.toml".into()],
                        },
                    );
                }
            }
        }
        for (file, manager) in [("poetry.lock", "poetry"), ("uv.lock", "uv")] {
            if text(dir, file).is_some() {
                add(&mut result.package_managers, manager, file);
            }
        }
        if let Some(requirements) = text(dir, "requirements.txt") {
            add(&mut result.languages, "Python", "requirements.txt");
            categories.push("python");
            for (name, display, category) in [
                ("django", "Django", "web"),
                ("fastapi", "FastAPI", "server"),
                ("flask", "Flask", "server"),
            ] {
                if requirements.lines().any(|line| {
                    line.trim()
                        .to_ascii_lowercase()
                        .split(['=', '<', '>', '[', ';', ' '])
                        .next()
                        == Some(name)
                }) {
                    add(&mut result.frameworks, display, "requirements.txt");
                    categories.push(category);
                }
            }
        }
        for (file, language, category, manager) in [
            ("go.mod", "Go", "go", "go"),
            ("pom.xml", "Java", "java", "maven"),
            ("build.gradle", "Java", "java", "gradle"),
            ("build.gradle.kts", "Kotlin", "java", "gradle"),
            ("CMakeLists.txt", "C++", "native", "cmake"),
        ] {
            if text(dir, file).is_some() {
                add(&mut result.languages, language, file);
                add(&mut result.package_managers, manager, file);
                categories.push(category);
            }
        }
        if let Ok(entries) = dir.entries() {
            for entry in entries.take(512).flatten() {
                let path = entry.file_name().to_string_lossy().into_owned();
                let language = if path.ends_with(".csproj") {
                    Some("C#")
                } else if path.ends_with(".fsproj") {
                    Some("F#")
                } else {
                    None
                };
                if let Some(language) = language.filter(|_| text(dir, &path).is_some()) {
                    add(&mut result.languages, language, &path);
                    add(&mut result.package_managers, "dotnet", &path);
                    categories.push("dotnet");
                }
            }
        }
        for (file, name) in [
            ("pnpm-workspace.yaml", "pnpm"),
            ("nx.json", "Nx"),
            ("turbo.json", "Turborepo"),
        ] {
            if text(dir, file).is_some() {
                add(&mut result.structure, "monorepo", file);
                if name == "pnpm" {
                    add(&mut result.package_managers, name, file);
                } else {
                    add(&mut result.frameworks, name, file);
                }
                categories.push("monorepo");
            }
        }
        if text(dir, "index.html").is_some() {
            add(&mut result.languages, "HTML", "index.html");
            categories.push("web");
        }
        let ecosystems = result
            .languages
            .iter()
            .map(|f| match f.name.as_str() {
                "JavaScript" | "TypeScript" => "js",
                other => other,
            })
            .collect::<std::collections::BTreeSet<_>>();
        if ecosystems.len() > 1 && ecosystems.contains("Python") {
            add(&mut result.structure, "monorepo", "pyproject.toml");
        }
        result.category = if !result.structure.is_empty() {
            "monorepo"
        } else if let Some(category) = categories.last() {
            category
        } else if text(dir, "README.md").is_some() || dir.is_dir("docs") {
            "docs"
        } else {
            "general"
        }
        .into();
        for facts in [
            &mut result.languages,
            &mut result.frameworks,
            &mut result.package_managers,
            &mut result.test_tools,
            &mut result.structure,
        ] {
            facts.sort_by(|a, b| a.name.cmp(&b.name));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tracked_stage_two_corpus() {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/project-detection");
        let cases: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("expected.json")).unwrap()).unwrap();
        for case in cases.as_array().unwrap() {
            let id = case["id"].as_str().unwrap();
            let profile = ProjectProfile::detect(&root.join(id));
            assert_eq!(profile.category, case["expected_kind"], "{id}");
            for (file, content) in case["files"].as_object().unwrap() {
                assert_eq!(
                    std::fs::read_to_string(root.join(id).join(file)).unwrap(),
                    content.as_str().unwrap(),
                    "unchanged {id}/{file}"
                );
            }
        }
    }
    #[test]
    fn scripts_locks_and_tools_are_facts_only() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("package.json"),r#"{"dependencies":{"next":"16"},"devDependencies":{"vitest":"1"},"scripts":{"test":"touch forbidden","build":"next build"}}"#).unwrap();
        std::fs::write(dir.path().join("pnpm-lock.yaml"), "").unwrap();
        let profile = ProjectProfile::detect(dir.path());
        assert_eq!(profile.commands["test"].name, "pnpm run test");
        assert_eq!(profile.test_tools[0].name, "vitest");
        assert!(!dir.path().join("forbidden").exists());
        assert_eq!(profile.project_kind(), "coding");
    }
    #[test]
    fn invalid_and_oversize_markers_do_not_invent_facts() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("package.json"), "{").unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "x".repeat(1_048_577)).unwrap();
        assert_eq!(ProjectProfile::detect(dir.path()).project_kind(), "general");
    }
    #[cfg(unix)]
    #[test]
    fn external_symlinks_are_not_read() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("package.json"), "{}").unwrap();
        std::os::unix::fs::symlink(
            outside.path().join("package.json"),
            dir.path().join("package.json"),
        )
        .unwrap();
        assert_eq!(ProjectProfile::detect(dir.path()).project_kind(), "general");
    }
}
