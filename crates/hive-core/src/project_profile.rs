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

/// Documentation wins only when no code category is present.
fn pick_category(categories: &[&str]) -> Option<&'static str> {
    let has_code = categories.iter().any(|value| {
        CATEGORY_PRECEDENCE
            .iter()
            // A bare package.json is generic "coding"; only a specific code category outranks docs.
            .any(|category| {
                category == value && !matches!(*category, "monorepo" | "docs" | "coding")
            })
    });
    CATEGORY_PRECEDENCE
        .iter()
        .find(|category| (**category != "docs" || !has_code) && categories.contains(category))
        .copied()
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

struct Marker {
    file: &'static str,
    language: &'static str,
    framework: &'static str,
    manager: &'static str,
    category: &'static str,
    confidence: &'static str,
}

// Specific frameworks outrank generic language/configuration markers.
const CATEGORY_PRECEDENCE: &[&str] = &[
    "monorepo", "docs", "desktop", "server", "cli", "web", "dotnet", "java", "native", "rust",
    "go", "python", "coding",
];
const MARKERS: &[Marker] = &[
    Marker {
        file: "next.config.ts",
        language: "",
        framework: "Next.js",
        manager: "",
        category: "web",
        confidence: "high",
    },
    Marker {
        file: "next.config.js",
        language: "",
        framework: "Next.js",
        manager: "",
        category: "web",
        confidence: "high",
    },
    Marker {
        file: "next.config.mjs",
        language: "",
        framework: "Next.js",
        manager: "",
        category: "web",
        confidence: "high",
    },
    Marker {
        file: "vite.config.ts",
        language: "",
        framework: "Vite",
        manager: "",
        category: "web",
        confidence: "high",
    },
    Marker {
        file: "vite.config.js",
        language: "",
        framework: "Vite",
        manager: "",
        category: "web",
        confidence: "high",
    },
    Marker {
        file: "svelte.config.js",
        language: "",
        framework: "Svelte",
        manager: "",
        category: "web",
        confidence: "high",
    },
    Marker {
        file: "astro.config.mjs",
        language: "",
        framework: "Astro",
        manager: "",
        category: "web",
        confidence: "high",
    },
    Marker {
        file: "angular.json",
        language: "",
        framework: "Angular",
        manager: "",
        category: "web",
        confidence: "high",
    },
    Marker {
        file: "tsconfig.json",
        language: "TypeScript",
        framework: "",
        manager: "",
        category: "coding",
        confidence: "high",
    },
    Marker {
        file: "go.mod",
        language: "Go",
        framework: "",
        manager: "go",
        category: "go",
        confidence: "high",
    },
    Marker {
        file: "pom.xml",
        language: "Java",
        framework: "",
        manager: "maven",
        category: "java",
        confidence: "high",
    },
    Marker {
        file: "build.gradle",
        language: "Java",
        framework: "",
        manager: "gradle",
        category: "java",
        confidence: "high",
    },
    Marker {
        file: "build.gradle.kts",
        language: "Kotlin",
        framework: "",
        manager: "gradle",
        category: "java",
        confidence: "high",
    },
    Marker {
        file: "CMakeLists.txt",
        language: "C++",
        framework: "",
        manager: "cmake",
        category: "native",
        confidence: "high",
    },
    Marker {
        file: "Gemfile",
        language: "Ruby",
        framework: "",
        manager: "bundler",
        category: "coding",
        confidence: "high",
    },
    Marker {
        file: "composer.json",
        language: "PHP",
        framework: "",
        manager: "composer",
        category: "coding",
        confidence: "high",
    },
    Marker {
        file: "Package.swift",
        language: "Swift",
        framework: "",
        manager: "swift",
        category: "coding",
        confidence: "high",
    },
    Marker {
        file: "pubspec.yaml",
        language: "Dart",
        framework: "",
        manager: "pub",
        category: "coding",
        confidence: "high",
    },
    Marker {
        file: "deno.json",
        language: "TypeScript",
        framework: "",
        manager: "deno",
        category: "coding",
        confidence: "high",
    },
    Marker {
        file: "deno.jsonc",
        language: "TypeScript",
        framework: "",
        manager: "deno",
        category: "coding",
        confidence: "high",
    },
    Marker {
        file: "mkdocs.yml",
        language: "",
        framework: "MkDocs",
        manager: "",
        category: "docs",
        confidence: "high",
    },
    Marker {
        file: "mkdocs.yaml",
        language: "",
        framework: "MkDocs",
        manager: "",
        category: "docs",
        confidence: "high",
    },
    Marker {
        file: "docusaurus.config.ts",
        language: "",
        framework: "Docusaurus",
        manager: "",
        category: "docs",
        confidence: "high",
    },
    Marker {
        file: "docusaurus.config.js",
        language: "",
        framework: "Docusaurus",
        manager: "",
        category: "docs",
        confidence: "high",
    },
    Marker {
        file: "tauri.conf.json",
        language: "",
        framework: "Tauri",
        manager: "",
        category: "desktop",
        confidence: "high",
    },
    Marker {
        file: "index.html",
        language: "HTML",
        framework: "",
        manager: "",
        category: "web",
        confidence: "high",
    },
];

fn entries(dir: &Dir) -> Vec<String> {
    let mut names = dir
        .entries()
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    names.sort();
    names.truncate(512);
    names
}

impl ProjectProfile {
    #[must_use]
    pub fn project_kind(&self) -> &str {
        if self.category == "docs"
            && self
                .frameworks
                .iter()
                .any(|fact| matches!(fact.name.as_str(), "MkDocs" | "Docusaurus"))
        {
            "docs"
        } else if matches!(self.category.as_str(), "general" | "docs" | "") {
            "general"
        } else {
            "coding"
        }
    }

    /// Infer facts from root and one-level marker files without commands or writes.
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
        let mut result = Self::detect_root(dir);
        let mut categories = if result.project_kind() == "general" {
            Vec::new()
        } else {
            vec![result.category.clone()]
        };
        for name in entries(dir) {
            if name.starts_with('.')
                || matches!(
                    name.as_str(),
                    "node_modules"
                        | "target"
                        | "vendor"
                        | "public"
                        | "dist"
                        | "build"
                        | "out"
                        | "site"
                        | "_site"
                        | "static"
                )
            {
                continue;
            }
            let Ok(child) = dir.open_dir(&name) else {
                continue;
            };
            let nested = Self::detect_root(&child);
            if nested.project_kind() == "general" {
                continue;
            }
            categories.push(nested.category);
            for (destination, source) in [
                (&mut result.languages, nested.languages),
                (&mut result.frameworks, nested.frameworks),
                (&mut result.package_managers, nested.package_managers),
                (&mut result.test_tools, nested.test_tools),
                (&mut result.structure, nested.structure),
            ] {
                for fact in source {
                    let existed = destination.iter().any(|item| item.name == fact.name);
                    for evidence in fact.evidence {
                        add(destination, &fact.name, &format!("{name}/{evidence}"));
                    }
                    if !existed {
                        if let Some(added) =
                            destination.iter_mut().find(|item| item.name == fact.name)
                        {
                            added.confidence.clone_from(&fact.confidence);
                        }
                    }
                }
                destination.sort_by(|a, b| a.name.cmp(&b.name));
            }
        }
        let names: Vec<&str> = categories.iter().map(String::as_str).collect();
        result.category = pick_category(&names).map_or(result.category, str::to_owned);
        result
    }

    #[allow(clippy::too_many_lines)]
    fn detect_root(dir: &Dir) -> Self {
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
                "coding"
            });
            // An empty workspace list declares no member packages.
            let members = &package["workspaces"];
            if members.as_array().is_some_and(|list| !list.is_empty())
                || members["packages"]
                    .as_array()
                    .is_some_and(|list| !list.is_empty())
            {
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
                        ("@docusaurus/core", "Docusaurus"),
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
                    if deps.contains_key("@docusaurus/core") {
                        categories.push("docs");
                    }
                    if [
                        "next",
                        "vite",
                        "react",
                        "vue",
                        "svelte",
                        "astro",
                        "@angular/core",
                        "typescript",
                    ]
                    .iter()
                    .any(|name| deps.contains_key(*name))
                    {
                        categories.push("web");
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
                            format!("js:{name}"),
                            Fact {
                                name: format!("{manager} run {name}"),
                                confidence: result.package_managers[0].confidence.clone(),
                                evidence: vec!["package.json".into()],
                            },
                        );
                    }
                }
            }
        }
        for marker in MARKERS {
            if text(dir, marker.file).is_none() {
                continue;
            }
            for (facts, name) in [
                (&mut result.languages, marker.language),
                (&mut result.frameworks, marker.framework),
                (&mut result.package_managers, marker.manager),
            ] {
                if !name.is_empty() {
                    add(facts, name, marker.file);
                    if let Some(fact) = facts.iter_mut().find(|fact| fact.name == name) {
                        fact.confidence = marker.confidence.into();
                    }
                }
            }
            categories.push(marker.category);
        }
        if let Some(cargo) =
            text(dir, "Cargo.toml").and_then(|s| toml::from_str::<toml::Value>(&s).ok())
        {
            add(&mut result.languages, "Rust", "Cargo.toml");
            add(&mut result.package_managers, "cargo", "Cargo.toml");
            categories.push("rust");
            if cargo
                .get("workspace")
                .and_then(|workspace| workspace.get("members"))
                .and_then(toml::Value::as_array)
                .is_some_and(|members| !members.is_empty())
            {
                add(&mut result.structure, "monorepo", "Cargo.toml");
            }
            for (key, command) in [
                ("build", "cargo build"),
                ("test", "cargo test"),
                ("lint", "cargo clippy"),
            ] {
                result.commands.insert(
                    format!("cargo:{key}"),
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
                        format!("python:{key}"),
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
            if dir.metadata(file).is_ok_and(|metadata| metadata.is_file()) {
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
        for path in entries(dir) {
            let extension = Path::new(&path)
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("");
            let (language, manager, category) = if extension.eq_ignore_ascii_case("csproj") {
                ("C#", "dotnet", "dotnet")
            } else if extension.eq_ignore_ascii_case("fsproj") {
                ("F#", "dotnet", "dotnet")
            } else if extension.eq_ignore_ascii_case("sln") {
                ("", "dotnet", "dotnet")
            } else if extension.eq_ignore_ascii_case("tf") {
                ("HCL", "terraform", "coding")
            } else if extension == "c" && dir.metadata("Makefile").is_ok_and(|m| m.is_file()) {
                ("C", "make", "native")
            } else {
                continue;
            };
            if text(dir, &path).is_some() {
                if !language.is_empty() {
                    add(&mut result.languages, language, &path);
                }
                add(&mut result.package_managers, manager, &path);
                categories.push(category);
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
        if let Some(py) =
            text(dir, "pyproject.toml").and_then(|value| toml::from_str::<toml::Value>(&value).ok())
        {
            if py
                .get("tool")
                .and_then(|tool| tool.get("uv"))
                .and_then(|uv| uv.get("workspace"))
                .is_some()
            {
                add(&mut result.structure, "monorepo", "pyproject.toml");
            }
        }
        // A build tool alone does not prove which language the code uses.
        for fact in &mut result.languages {
            if fact.evidence.iter().all(|path| {
                matches!(
                    path.as_str(),
                    "CMakeLists.txt" | "build.gradle" | "build.gradle.kts"
                )
            }) {
                fact.confidence = "low".into();
            }
        }
        result.category = if !result.structure.is_empty() {
            "monorepo"
        } else if let Some(category) = pick_category(&categories) {
            category
        } else if text(dir, "README.md").is_some() || dir.is_dir("docs") {
            "docs" // Low-confidence documentation hint: project_kind still requires a list choice.
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
            assert_eq!(
                profile.category,
                case.get("reviewed_expected_category")
                    .unwrap_or(&case["expected_kind"])
                    .as_str()
                    .unwrap(),
                "{id}"
            );
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
    fn weak_readme_requires_choice_and_nested_markers_supply_real_evidence() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("README.md"), "# Notes").unwrap();
        assert_eq!(ProjectProfile::detect(dir.path()).project_kind(), "general");
        std::fs::create_dir(dir.path().join("backend")).unwrap();
        std::fs::write(
            dir.path().join("backend/Cargo.toml"),
            "[package]\nname='server'\n",
        )
        .unwrap();
        let profile = ProjectProfile::detect(dir.path());
        assert_eq!(profile.project_kind(), "coding");
        assert_eq!(profile.languages[0].evidence, ["backend/Cargo.toml"]);
        assert!(profile.commands.is_empty()); // Nested commands require a different working directory.
    }

    #[test]
    fn precedence_commands_and_mixed_projects_do_not_invent_structure() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("package.json"), r#"{"dependencies":{"express":"5"},"scripts":{"test":"node test"},"packageManager":"pnpm@10"}"#).unwrap();
        std::fs::write(dir.path().join("tsconfig.json"), "{}").unwrap();
        assert_eq!(ProjectProfile::detect(dir.path()).category, "server");
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname='native'\n").unwrap();
        std::fs::write(dir.path().join("requirements.txt"), "Django==5").unwrap();
        let profile = ProjectProfile::detect(dir.path());
        assert!(profile.structure.is_empty());
        assert_eq!(profile.commands["js:test"].name, "pnpm run test");
        assert_eq!(profile.commands["cargo:test"].name, "cargo test");
        for fact in profile.languages.iter().chain(&profile.structure) {
            assert!(fact
                .evidence
                .iter()
                .all(|path| dir.path().join(path).exists()));
        }
    }

    #[test]
    fn empty_workspaces_docs_beside_code_and_output_folders_do_not_mislead() {
        // Empty workspace declarations are not a monorepo.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]
name='one'
[workspace]
",
        )
        .unwrap();
        assert!(ProjectProfile::detect(dir.path()).structure.is_empty());
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("package.json"), r#"{"workspaces":[]}"#).unwrap();
        assert!(ProjectProfile::detect(dir.path()).structure.is_empty());
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[workspace]
members=['crates/a']
",
        )
        .unwrap();
        assert_eq!(ProjectProfile::detect(dir.path()).category, "monorepo");
        // A Python project with an mkdocs site stays a coding project.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("pyproject.toml"),
            "[project]
name='tool'
",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("mkdocs.yml"),
            "site_name: Tool
",
        )
        .unwrap();
        assert_ne!(ProjectProfile::detect(dir.path()).category, "docs");
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("mkdocs.yml"),
            "site_name: Notes
",
        )
        .unwrap();
        assert_eq!(ProjectProfile::detect(dir.path()).category, "docs");
        // A documentation site's package.json alone does not make it a code project.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"dependencies":{"@docusaurus/core":"3"}}"#,
        )
        .unwrap();
        assert_eq!(ProjectProfile::detect(dir.path()).category, "docs");
        // A generated or static output page does not turn notes into code.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("README.md"),
            "# Notes
",
        )
        .unwrap();
        std::fs::create_dir(dir.path().join("public")).unwrap();
        std::fs::write(dir.path().join("public/index.html"), "<html></html>").unwrap();
        assert_eq!(ProjectProfile::detect(dir.path()).category, "docs");
    }

    #[test]
    fn build_tools_alone_do_not_claim_a_language_with_high_confidence() {
        for file in ["CMakeLists.txt", "build.gradle"] {
            let dir = tempfile::tempdir().unwrap();
            std::fs::write(dir.path().join(file), "").unwrap();
            let profile = ProjectProfile::detect(dir.path());
            assert!(
                profile
                    .languages
                    .iter()
                    .all(|fact| fact.confidence == "low"),
                "{file}"
            );
            assert!(
                profile
                    .package_managers
                    .iter()
                    .all(|fact| fact.confidence == "high"),
                "{file}"
            );
        }
    }

    #[test]
    fn large_python_locks_and_sorted_entry_limit() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["poetry.lock", "uv.lock"] {
            std::fs::write(dir.path().join(name), vec![0xff; 1_048_577]).unwrap();
        }
        // Reverse creation order must not affect the deterministic first 512 paths.
        for n in (0..520).rev() {
            std::fs::write(dir.path().join(format!("z{n:04}.txt")), "").unwrap();
        }
        std::fs::write(dir.path().join("a.csproj"), "<Project/>").unwrap();
        let profile = ProjectProfile::detect(dir.path());
        assert!(profile.languages.iter().any(|fact| fact.name == "C#"));
        assert!(profile
            .package_managers
            .iter()
            .any(|fact| fact.name == "uv"));
        assert!(profile
            .package_managers
            .iter()
            .any(|fact| fact.name == "poetry"));
    }

    #[test]
    fn scripts_locks_and_tools_are_facts_only() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("package.json"),r#"{"dependencies":{"next":"16"},"devDependencies":{"vitest":"1"},"scripts":{"test":"touch forbidden","build":"next build"}}"#).unwrap();
        std::fs::write(dir.path().join("pnpm-lock.yaml"), "").unwrap();
        let profile = ProjectProfile::detect(dir.path());
        assert_eq!(profile.commands["js:test"].name, "pnpm run test");
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
