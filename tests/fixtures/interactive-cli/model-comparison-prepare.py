"""Prepare fresh, pinned M20 inputs; never launch a model."""
import argparse
import ast
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

import yaml

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
from scripts.test_artifacts import Run
from tests.conformance.support.harness import write_operational_user_setup

assert os.name == "nt", "This qualified transport uses private Windows profiles."
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--project-guard-off", action="store_true")
parser.add_argument("--project-medium", action="store_true")
options = parser.parse_args()
if options.project_medium and not options.project_guard_off:
    parser.error("--project-medium requires the explicit --project-guard-off comparison")
BASE = ROOT / ("tests/work/m20-project-medium" if options.project_medium
               else "tests/work/m20-project-guard-off-1d01e7d0" if options.project_guard_off
               else "tests/work/m20-comparison-1d01e7d0")
assert not BASE.exists(), "Never replace an earlier trial."
BASELINE = ROOT / "tests/work/legacy-file-normalize-0140/public-package/bin/hive.exe"
CANDIDATE = ROOT / "tests/work/m19-candidate-1d01e7d0/hive.exe"
PINS = {"0.13.1": (BASELINE, "e34e5b7486feb6adc52f5b0ca59c01da303a14af2ecd397b1c8e60c9ecab7824"),
        "0.14.0": (CANDIDATE, "3a34631483b7fa1f660a82fa811e5c6b10ba722b35a3b07cc60b77ce17dd263f")}
OLD_COMMIT = "c37882ca7a66e895961f630a3d40c584d910279a"
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
for binary, expected in PINS.values():
    assert sha(binary) == expected
git = Path(shutil.which("git")).resolve()
tree = ast.parse((ROOT / "tests/conformance/integration/test_connected_setup_lifecycle.py").read_text(encoding="utf-8"))
klass = next(n for n in tree.body if isinstance(n, ast.ClassDef) and n.name == "ConnectedSetupLifecycleConformance")
method = next(n for n in klass.body if isinstance(n, ast.FunctionDef) and n.name == "_write_fake_host")
stub = next(n.value for n in ast.walk(method) if isinstance(n, ast.Constant) and isinstance(n.value, str) and n.value.startswith("import json"))
project_choices = yaml.safe_load(subprocess.check_output([str(git), "show", OLD_COMMIT + ":tests/fixtures/setup/answers-no-role-no-hook.yml"], text=True))
project_choices.pop("schema_version")
project_choices["project_name"] = "luna-quick"
project_choices["knowledge_exclude_paths"] = yaml.safe_load((ROOT / "tests/work/m19-luna-project-role-complete-command/project/.hive/setup-answers.yml").read_text(encoding="utf-8"))["knowledge_exclude_paths"]
common_user = {"interface_language": "ko", "selected_hosts": ["codex"],
    "wiki": {"enabled": True, "language": "ko", "backend": "markdown"},
    "profile": {"contexts": ["non-developer"]}, "persona": {"id": "balanced"},
    "skills": {"mode": "all"}, "judge_invocation": "explicit", "update_check": {"enabled": False}}
record = Run("M20 원본/후보의 독립 고정 입력 준비", [sys.executable, str(Path(__file__).resolve()), *sys.argv[1:]],
             paths=[BASE.relative_to(ROOT).as_posix()])
trials = []
code = 1
try:
    BASE.mkdir()
    for scope in (["project"] if options.project_guard_off else ["user", "project"]):
        for version, (binary, binary_hash) in PINS.items():
            for sample in [1, 2]:
                trial = BASE / f"{scope}-{version}-{sample}"
                trial.mkdir()
                for name in ["user", "project", "profile", "tmp", "closed-bin", "prefix", "npm-cache", "skill"]:
                    (trial / name).mkdir()
                (trial / scope / "README.txt").write_bytes(b"Synthetic user note. Preserve exactly.\n")
                skill_name = "user-setup" if scope == "user" else "project-setup"
                source = "harness/skills/" + skill_name
                if version == "0.13.1":
                    files = subprocess.check_output([str(git), "ls-tree", "-r", "--name-only", OLD_COMMIT, "--", source], text=True).splitlines()
                    for name in files:
                        target = trial / "skill" / Path(name).relative_to(source)
                        target.parent.mkdir(parents=True, exist_ok=True)
                        target.write_bytes(subprocess.check_output([str(git), "show", OLD_COMMIT + ":" + name]))
                else:
                    for path in (ROOT / source).rglob("*"):
                        if path.is_file():
                            target = trial / "skill" / path.relative_to(ROOT / source)
                            target.parent.mkdir(parents=True, exist_ok=True)
                            shutil.copyfile(path, target)
                shutil.copyfile(ROOT / "tests/fixtures/interactive-cli/model-trial-tool.py", trial / "tool.py")
                shutil.copyfile(ROOT / "tests/fixtures/setup/capabilities-codex-host-native.json", trial / "capabilities.json")
                user_before, fragments = {}, {}
                if scope == "project":
                    write_operational_user_setup(trial / "user")
                    if options.project_guard_off:
                        settings_path = trial / "user/.hive/config/user-setup.yml"
                        settings = yaml.safe_load(settings_path.read_text(encoding="utf-8"))
                        settings["usage_guard"]["enabled"] = False
                        settings["usage_guard"]["codexbar_fallback_enabled"] = False
                        settings_path.write_text(yaml.safe_dump(settings, sort_keys=False), encoding="utf-8")
                    user_before[".hive/config/user-setup.yml"] = sha(trial / "user/.hive/config/user-setup.yml")
                    for name, content in {"AGENTS.md": "Synthetic user instruction. Preserve exactly.\n", ".gitignore": "user-cache/\n"}.items():
                        (trial / "project" / name).write_text(content, encoding="utf-8")
                        fragments["project/" + name] = (trial / "project" / name).read_bytes().decode("utf-8")
                    env = {"PATH": str(trial / "closed-bin"), "HOME": str(trial / "profile"), "USERPROFILE": str(trial / "profile"),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": str(trial / "empty-git-config"),
                           "SystemRoot": os.environ["SystemRoot"]}
                    subprocess.run([str(git), "-c", "init.defaultBranch=fixture", "init", "--quiet", str(trial / "project")], env=env, check=True, capture_output=True)
                    choices = project_choices
                    expected_values = {"/" + k: v for k, v in choices.items()}
                    fields = ["/project_name", "/project_kind", "/primary_host"]
                else:
                    choices = {**common_user, "usage_guard": {
                        "enabled" if version == "0.13.1" else "enforcement_enabled": False,
                        "stop_remaining_percent": 20, "codexbar_fallback_enabled": False, "discord": {"enabled": False}}}
                    expected_values = {"/" + k: v for k, v in common_user.items()}
                    expected_values.update({"/usage_guard/" + ("enabled" if version == "0.13.1" else "enforcement_enabled"): False,
                        "/usage_guard/stop_remaining_percent": 20, "/usage_guard/codexbar_fallback_enabled": False,
                        "/usage_guard/discord/enabled": False})
                    fields = ["/interface_language", "/selected_hosts"]
                    (trial / "closed-bin/codex.py").write_text(stub.replace("__HIVE_TEST_HOST__", repr("codex")).replace("__HIVE_TEST_PRODUCT_VERSION__", repr(version)), encoding="utf-8")
                    (trial / "closed-bin/codex.cmd").write_text(f'@"{sys.executable}" "%~dp0codex.py" %*\r\n', encoding="utf-8")
                (trial / "fixed-choices.json").write_text(json.dumps(choices, indent=2), encoding="utf-8")
                readable = ["fixed-choices.json", "capabilities.json", "user/.hive/config/user-setup.yml"]
                if scope == "project":
                    readable += ["project/" + name for name in ["README.txt", "AGENTS.md", "CLAUDE.md", "GEMINI.md", ".gitignore", "package.json", "Cargo.toml", "pyproject.toml", ".hive/setup-answers.yml", ".hive/config/harness.toml", ".hive/config/active-skills.yml"]]
                config = {"root": str(trial), "comparison": True, "comparison_version": version, "sample": sample,
                    "binary": str(binary), "binary_sha256": binary_hash, "product_commit": OLD_COMMIT if version == "0.13.1" else "1d01e7d0",
                    "source_commit": subprocess.check_output([str(git), "rev-parse", "HEAD"], text=True).strip(),
                    "model": "gpt-6-luna", "reasoning_effort": "medium" if scope == "user" or options.project_medium else "low",
                    "scenario": scope + "-quick", "setup_scope": scope, "expected_values": expected_values,
                    "user_before": user_before, "preserved_fragments": fragments, "agent_fields": fields,
                    "readable_paths": readable, "git_binary": str(git), "git_sha256": sha(git),
                    "requested_speed_mode": "host-default-non-fast (user-confirmed; no Fast request or setting change)",
                    "speed": "measured throughput unavailable; tool priority metadata is not interpreted as UI Fast",
                    "token_measurements_available": False, "cache_conditions": "new conversation and private filesystem per trial; provider prompt cache unknown",
                    "optional_features": "explicitly deferred; no activation or inferred answer",
                    "project_guard": "disabled with fallback disabled" if options.project_guard_off else "original fixture defaults",
                    "input_sha256": {p.relative_to(trial).as_posix(): sha(p) for p in trial.rglob("*") if p.is_file() and ".git" not in p.relative_to(trial).parts},
                    "python": sys.executable}
                (trial / "config.json").write_text(json.dumps(config, indent=2), encoding="utf-8")
                trials.append({"path": trial.relative_to(ROOT).as_posix(), "version": version, "scope": scope, "sample": sample,
                               "config_sha256": sha(trial / "config.json"), "choices_sha256": sha(trial / "fixed-choices.json")})
    (BASE / "manifest.json").write_text(json.dumps({"trials": trials, "requested_mode": "host-default-non-fast", "model_runs": 0}, indent=2), encoding="utf-8")
    code = 0
finally:
    record.finish(code, details={"trials": trials, "scope": "Preparation only. Public immutable13.1 Skill/Git bytes and original image; fixed14 image/current Skill; fresh private fixtures. No model, real app/service or cost-result claim."})
print(json.dumps({"root": str(BASE), "trial_count": len(trials), "model_runs": 0}))
