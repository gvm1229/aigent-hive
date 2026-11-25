use super::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const NONE: [&str; 0] = [];

fn top() -> Vec<&'static str> {
    top_level_candidates()
}

fn options_of(name: &str) -> Vec<&'static str> {
    let mut options = find(name).unwrap().options.to_vec();
    options.extend(["--help", "--output"]);
    options
}

#[test]
fn close_typos_get_real_candidates() {
    assert_eq!(suggest("instal", &top()), ["install"]);
    assert_eq!(suggest("knowlege", &top())[0], "knowledge");
    assert_eq!(suggest("stup", &top())[0], "setup");
    assert_eq!(suggest("help", &top()), ["--help"]);
    assert_eq!(suggest("know", &top()), ["knowledge"]);
    assert_eq!(suggest("--dryrun", &options_of("install")), ["--dry-run"]);
    assert_eq!(suggest("--hots", &options_of("install"))[0], "--host");
    assert_eq!(
        suggest("retreive", find("knowledge").unwrap().subcommands),
        ["retrieve"]
    );
    assert_eq!(
        suggest("upgade", find("project").unwrap().subcommands),
        ["upgrade"]
    );
}

#[test]
fn nothing_close_means_no_suggestion() {
    assert_eq!(suggest("xyzzy", &top()), NONE);
    assert_eq!(suggest("unknown-action", &top()), NONE);
    assert_eq!(suggest("--flags", &options_of("install")), NONE);
    assert_eq!(suggest("", &top()), NONE);
    // Four typed letters allow two edits, not three.
    assert_eq!(suggest("abcd", &["abxy", "axyz"]), ["abxy"]);
    // Three typed letters allow one edit.
    assert_eq!(suggest("abc", &["axy"]), NONE);
    assert_eq!(suggest("abc", &["abx"]), ["abx"]);
    // A single letter is not a useful prefix.
    assert_eq!(suggest("k", &top()), NONE);
}

#[test]
fn at_most_three_candidates_closest_first() {
    let found = suggest("re", &top());
    assert!(
        !found.is_empty() && found.len() <= MAX_SUGGESTIONS,
        "{found:?}"
    );
    assert!(found.iter().all(|name| name.starts_with("re")), "{found:?}");
    assert_eq!(
        suggest("instal", &["install", "instal1", "instals", "insta"]).len(),
        3
    );
}

#[test]
fn examples_include_required_options_and_follow_the_subcommand() {
    assert_eq!(
        example_for("install", None),
        Some("hive install --scope user --hosts codex,claude --dry-run")
    );
    assert!(example_for("knowledge", Some("retrieve"))
        .unwrap()
        .starts_with("hive knowledge retrieve --user-root"));
    assert_eq!(
        example_for("knowledge", Some("subjects")),
        example_for("knowledge", None)
    );
    for command in COMMANDS {
        let example = command.examples.first().unwrap();
        assert!(
            example.starts_with(&format!("hive {}", command.name)),
            "{}",
            command.name
        );
        assert!(width(example) <= COLUMNS - 8, "{example}");
    }
}

#[test]
fn rejection_text_names_input_suggestion_example_and_help() {
    let rejection = Rejection {
        problem: Problem::Option,
        command: find("install"),
        input: "--dryrun",
        suggestions: vec!["--dry-run"],
        example: Some("hive install --scope user --hosts codex,claude --dry-run".to_owned()),
        help: help_command("install"),
    };
    let ko = rejection_text(&rejection, InterfaceLanguage::Ko);
    assert_eq!(
        ko,
        "처리 실패: hive install에 없는 옵션입니다: --dryrun\n혹시 이것인가요? --dry-run\n\
         예시: hive install --scope user --hosts codex,claude --dry-run\n\
         다음 단계: hive install --help 로 옵션과 예시를 확인하세요.\n"
    );
    let en = rejection_text(&rejection, InterfaceLanguage::En);
    assert!(en.contains("Did you mean? --dry-run\n"), "{en}");
    assert!(en.contains("Next: see hive install --help"), "{en}");
    let none = Rejection {
        problem: Problem::Command,
        command: None,
        input: "xyzzy",
        suggestions: Vec::new(),
        example: None,
        help: "hive --help".to_owned(),
    };
    let text = rejection_text(&none, InterfaceLanguage::Ko);
    assert!(!text.contains("혹시"), "{text}");
    assert!(text.contains("hive --help"), "{text}");
}

fn source_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn rust_files(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_dir() {
        for entry in std::fs::read_dir(path).unwrap() {
            rust_files(&entry.unwrap().path(), files);
        }
    } else if path.extension().is_some_and(|extension| extension == "rs") {
        files.push(path.to_owned());
    }
}

/// Every `"--name"` string literal in the given source text.
fn option_literals(text: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for (index, _) in text.match_indices("\"--") {
        let rest = &text[index + 1..];
        if let Some(end) = rest[1..].find('"') {
            let literal = &rest[..=end];
            if looks_like_option(literal) {
                found.insert(literal.to_owned());
            }
        }
    }
    found
}

/// Parser sources and the commands they serve. Ignored literals are arguments for other
/// programs (git, npm, host CLIs) or rejected test inputs, never Hive options.
#[rustfmt::skip]
const SOURCES: &[(&[&str], &[&str], &[&str])] = &[
    (&["main.rs"], &["setup", "update", "usage", "advanced", "hook"], &[]),
    (&["user_setup.rs", "user_setup"], &["setup"], &[]),
    (&["user_install.rs", "user_install"], &["install", "uninstall", "update"],
     &["--json", "--version", "--full", "--purge", "--operation"]),
    (&["update.rs", "update_activation.rs", "update_activation", "update_discovery.rs", "update_bridge"],
     &["update", "release"], &["--global", "--prefix", "--ignore-scripts", "--interaction"]),
    (&["usage_control.rs", "usage_control", "usage_install.rs", "usage_monitor.rs", "usage_monitor"],
     &["usage"], &["--cask", "--incident", "--unknown"]),
    (&["knowledge.rs", "knowledge", "knowledge_transfer.rs", "knowledge_scan.rs", "vector.rs",
       "vector_auth.rs", "vector_batch.rs", "vector_index.rs", "vector_query.rs"],
     &["knowledge", "source-wiki", "index"],
     &["--cached", "--exact", "--exclude-standard", "--local", "--nocapture", "--others",
       "--short", "--show-toplevel", "--version"]),
    (&["source_wiki.rs"], &["source-wiki"], &["--other"]),
    (&["custom_agent_cli.rs"], &["agent"], &[]),
    (&["discord.rs"], &["discord"], &[]),
    (&["judge.rs"], &["judge"], &[]),
    (&["korean.rs"], &["korean"], &[]),
    (&["loop_engineering.rs"], &["loop"], &[]),
    (&["orchestration.rs"], &["orchestration"], &[]),
    (&["policy.rs", "policy"], &["policy"], &[]),
    (&["project_upgrade.rs", "project_upgrade"], &["project"], &[]),
    (&["report.rs"], &["report"], &[]),
    (&["role.rs"], &["role"], &[]),
    (&["roster.rs"], &["roster"], &["--unknown"]),
    (&["run.rs", "run"], &["run"], &[]),
    (&["session.rs"], &["session"], &[]),
    (&["storage.rs"], &["storage", "prune"], &[]),
    (&["summon.rs"], &["summon"], &[]),
    (&["msg.rs", "msg"], &["msg", "stop"], &[]),
    // The usage provider adapters call codexbar and host CLIs, not Hive parsers.
    (&["usage.rs", "usage"], &[], &["--all-accounts", "--format", "--json-only", "--provider",
       "--source", "--stdio", "--version"]),
    (&["cli_help.rs", "cli_help", "display_options.rs", "gods.rs", "human_output.rs",
       "human_output", "user_directives.rs"], &[], &[]),
];

#[test]
fn registry_lists_every_option_the_parsers_accept() {
    let root = source_root();
    let mut covered = BTreeSet::new();
    for (paths, commands, ignored) in SOURCES {
        let mut files = Vec::new();
        for path in *paths {
            rust_files(&root.join(path), &mut files);
        }
        assert!(!files.is_empty(), "{paths:?}");
        let mut allowed: BTreeSet<&str> = GLOBAL_OPTIONS.iter().copied().collect();
        for name in *commands {
            allowed.extend(find(name).unwrap_or_else(|| panic!("{name}")).options);
        }
        for file in files {
            // The registry and its tests quote ignored literals on purpose.
            if file.starts_with(root.join("cli_help")) || file == root.join("cli_help.rs") {
                covered.insert(file);
                continue;
            }
            let literals = option_literals(&std::fs::read_to_string(&file).unwrap());
            for literal in literals.iter().filter(|l| !ignored.contains(&l.as_str())) {
                assert!(
                    known_anywhere(literal),
                    "{} accepts {literal}, missing from the registry",
                    file.display()
                );
                assert!(
                    commands.is_empty() || allowed.contains(literal.as_str()),
                    "{} parses {literal}; add it to {commands:?}",
                    file.display()
                );
            }
            covered.insert(file);
        }
    }
    let mut all = Vec::new();
    rust_files(&root, &mut all);
    for file in all {
        assert!(
            covered.contains(&file),
            "{} is not mapped in SOURCES",
            file.display()
        );
    }
}

fn dispatch_source() -> String {
    let main = std::fs::read_to_string(source_root().join("main.rs")).unwrap();
    let start = main.find("fn dispatch(arguments").unwrap();
    let end = start + main[start..].find("\nstruct UsageArguments").unwrap();
    let human = main.find("fn run_human(").unwrap();
    let human_end = human + main[human..].find("\nfn version_output()").unwrap();
    format!("{}{}", &main[start..end], &main[human..human_end])
}

#[test]
fn registry_lists_every_dispatched_command() {
    let source = dispatch_source();
    let mut words = BTreeSet::new();
    for (index, _) in source.match_indices("Some(\"") {
        let rest = &source[index + 6..];
        let word = &rest[..rest.find('"').unwrap()];
        words.insert(word.to_owned());
    }
    for (index, _) in source.match_indices(" | \"") {
        let rest = &source[index + 4..];
        let word = &rest[..rest.find('"').unwrap()];
        words.insert(word.to_owned());
    }
    // Arms that are not commands: setup's own subcommand, helper workers and flags.
    for word in words
        .iter()
        .filter(|word| !word.starts_with('-') && !word.starts_with("__"))
        .filter(|word| *word != "feature")
    {
        assert!(
            find(word).is_some(),
            "dispatched command {word} is not registered"
        );
    }
    for command in COMMANDS {
        assert!(
            words.contains(command.name),
            "registered command {} is not dispatched",
            command.name
        );
    }
}

#[test]
fn registry_subcommands_exist_in_the_sources() {
    let root = source_root();
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    let text: String = files
        .iter()
        .map(|file| std::fs::read_to_string(file).unwrap())
        .collect();
    for command in COMMANDS {
        for sub in command.subcommands {
            assert!(
                text.contains(&format!("\"{sub}\"")),
                "{} {sub} is not parsed anywhere",
                command.name
            );
        }
    }
}

fn all_help(language: InterfaceLanguage) -> Vec<(String, String)> {
    let mut helps = vec![("--help".to_owned(), top_help(language))];
    for arguments in [
        vec!["install", "--help"],
        vec!["setup", "--help"],
        vec!["update", "--help"],
        vec!["knowledge", "--help"],
        vec!["knowledge", "retrieve", "--help"],
        vec!["usage", "--help"],
        vec!["project", "upgrade", "--help"],
    ] {
        let arguments: Vec<String> = arguments.into_iter().map(str::to_owned).collect();
        let text = help_request(&arguments, language)
            .unwrap_or_else(|| panic!("{arguments:?} has no help"));
        helps.push((arguments.join(" "), text));
    }
    helps
}

#[test]
fn help_fits_eighty_columns_and_matches_the_snapshot() {
    let mut snapshot = String::new();
    for language in [InterfaceLanguage::Ko, InterfaceLanguage::En] {
        for (arguments, text) in all_help(language) {
            for line in text.lines() {
                assert!(width(line) <= COLUMNS, "{arguments}: {line}");
                assert_eq!(line, line.trim_end(), "{arguments}: trailing space");
            }
            let code = if language == InterfaceLanguage::Ko {
                "ko"
            } else {
                "en"
            };
            write!(snapshot, "=== hive {arguments} ({code})\n{text}\n").unwrap();
        }
    }
    // One final newline, not a blank line, at the end of the file.
    snapshot.pop();
    // The snapshot records the default build; optional features such as `notion-preview` add
    // their own syntax lines, which still pass the width checks above.
    if cfg!(not(feature = "notion-preview")) {
        if std::env::var_os("HIVE_WRITE_HELP_SNAPSHOT").is_some() {
            std::fs::write(source_root().join("cli_help/help.snap"), &snapshot).unwrap();
        }
        assert_eq!(snapshot, include_str!("help.snap"));
    }
}

#[test]
fn top_help_groups_every_command_once() {
    for language in [InterfaceLanguage::Ko, InterfaceLanguage::En] {
        let text = top_help(language);
        let mut last = 0;
        for (_, ko, en) in GROUPS {
            let title = if language == InterfaceLanguage::Ko {
                ko
            } else {
                en
            };
            let at = text
                .lines()
                .position(|line| line == title)
                .unwrap_or_else(|| panic!("{title}"));
            assert!(at > last, "{title} out of order");
            last = at;
        }
        for command in COMMANDS {
            let prefix = format!("  {} ", command.name);
            assert_eq!(
                text.lines()
                    .filter(|line| line.starts_with(&prefix))
                    .count(),
                1,
                "{}",
                command.name
            );
        }
    }
}

#[test]
fn main_command_help_has_tables_and_two_examples() {
    for language in [InterfaceLanguage::Ko, InterfaceLanguage::En] {
        let ko = language == InterfaceLanguage::Ko;
        for (arguments, text) in all_help(language).into_iter().filter(|(arguments, _)| {
            arguments != "--help" && arguments != "knowledge retrieve --help"
        }) {
            for heading in if ko {
                ["필수 옵션", "선택 옵션", "예시", "전체 형식"]
            } else {
                ["Required options", "Optional", "Examples", "Full syntax"]
            } {
                assert!(
                    text.lines().any(|line| line == heading),
                    "{arguments}: {heading}"
                );
            }
            let examples = text
                .split(if ko { "\n예시\n" } else { "\nExamples\n" })
                .nth(1)
                .unwrap()
                .lines()
                .take_while(|line| !line.is_empty())
                .filter(|line| line.starts_with("  hive "))
                .count();
            assert!(examples >= 2, "{arguments}: {examples} examples");
        }
    }
}

fn normalized(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn every_old_usage_line_stays_reachable_from_some_help() {
    let mut usage = crate::USAGE.to_owned();
    usage.push_str(crate::SETUP_USAGE);
    usage.push_str(&knowledge_usage());
    usage.push_str(crate::update::UPDATE_USAGE);
    for line in usage
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("hive "))
    {
        let words: Vec<&str> = line.split_whitespace().collect();
        let family = words[1];
        if family == "<god>" || line.ends_with("--help") {
            continue;
        }
        let arguments = vec![family.to_owned(), "--help".to_owned()];
        match help_request(&arguments, InterfaceLanguage::En) {
            Some(text) => {
                let sub = (family == "knowledge")
                    .then(|| {
                        help_request(
                            &["knowledge".into(), words[2].into(), "--help".into()],
                            InterfaceLanguage::En,
                        )
                    })
                    .flatten()
                    .unwrap_or_default();
                assert!(
                    normalized(&text).contains(&normalized(line))
                        || normalized(&sub).contains(&normalized(line)),
                    "{line}"
                );
            }
            // Other families keep their existing help path, which prints these lines.
            None => assert!(
                crate::family_help_request(&arguments).is_some()
                    || matches!(family, "source-wiki" | "hook" | "policy" | "discord"),
                "{line}"
            ),
        }
    }
}

#[test]
fn wrapping_keeps_every_word_and_indents_syntax() {
    let line = format!("    hive example {}", "--option <value> ".repeat(10));
    let wrapped = wrap(&line);
    assert!(wrapped.lines().count() > 1);
    assert!(wrapped.lines().skip(1).all(|l| l.starts_with("        ")));
    assert_eq!(normalized(&wrapped), normalized(&line));
}

#[test]
fn option_shaped_value_after_a_known_option_reaches_the_parser() {
    let arguments = |words: &[&str]| {
        words
            .iter()
            .map(|word| (*word).to_owned())
            .collect::<Vec<_>>()
    };
    assert!(precheck(&arguments(&["knowledge", "query", "--text", "--verbose"])).is_none());
    assert!(precheck(&arguments(&["install", "--scope", "user", "--flags"])).is_some());
}

#[test]
fn retired_top_level_commands_point_to_hive_advanced() {
    assert_eq!(MOVED_TO_ADVANCED, ["route", "prompt"]);
    for name in MOVED_TO_ADVANCED {
        assert!(find(name).is_none(), "{name} stays off the top level");
        assert!(find("advanced").unwrap().subcommands.contains(name));
        assert!(!top_level_candidates().contains(name));
    }
    assert_eq!(
        moved_example("route"),
        "hive advanced route --request <request.json>"
    );
    assert_eq!(
        moved_text("prompt", moved_example("prompt"), InterfaceLanguage::Ko),
        "처리 실패: hive prompt 명령은 hive advanced prompt로 옮겨졌습니다.\n\
         예시: hive advanced prompt validate --request <input.json> --result <result.json>\n\
         다음 단계: hive advanced --help 로 옵션과 예시를 확인하세요.\n"
    );
    assert_eq!(
        moved_text("route", moved_example("route"), InterfaceLanguage::En),
        "Failed. hive route moved to hive advanced route.\n\
         Example: hive advanced route --request <request.json>\n\
         Next: see hive advanced --help for options and examples.\n"
    );
    assert!(moved_command(&["usage".to_owned()]).is_none());
}
