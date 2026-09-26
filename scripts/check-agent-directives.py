#!/usr/bin/env python3
"""Check active Agent directive ownership, size budgets, and projection parity."""

from __future__ import annotations

import argparse
import json
import re
from collections import defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SOURCE_BASELINE = 66_849
CONSUMER_BASELINE = 13_856
SOURCE_FILES = tuple(sorted((ROOT / ".agents/directives").glob("*.md")))
CONSUMER_DIRECTIVES = tuple(sorted((ROOT / "harness/directives").glob("*.md")))
ROUTER_TARGETS = (
    ".agents/directives/00-editing-discipline.md",
    ".agents/directives/01-behavior.md",
    ".agents/directives/02-architecture.md",
    ".agents/directives/03-workflow.md",
    ".agents/directives/04-documentation-state.md",
    ".agents/directives/05-security-safety.md",
    ".agents/directives/06-session-coordination.md",
    ".agents/directives/07-installed-usage-guard.md",
    ".agents/directives/08-human-documentation-style.md",
)


def normalized_bullets(path: Path) -> list[str]:
    bullets: list[str] = []
    current = ""
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("- "):
            if current:
                bullets.append(" ".join(current.split()))
            current = line[2:]
        elif current and (line.startswith("  ") or not line.strip()):
            if line.strip():
                current += " " + line.strip()
        elif current:
            bullets.append(" ".join(current.split()))
            current = ""
    if current:
        bullets.append(" ".join(current.split()))
    return [bullet for bullet in bullets if len(bullet) >= 80]


def duplicate_findings(paths: tuple[Path, ...]) -> list[dict[str, object]]:
    owners: dict[str, list[str]] = defaultdict(list)
    for path in paths:
        for bullet in normalized_bullets(path):
            owners[bullet].append(path.relative_to(ROOT).as_posix())
    return [
        {"text": text, "paths": locations}
        for text, locations in sorted(owners.items())
        if len(set(locations)) > 1
    ]


def check_read_budgets(root: Path) -> tuple[dict[str, object], list[dict[str, object]]]:
    """Measure declared complete phase read sets, including conditional references."""
    metrics, failures = {}, []
    try:
        budget = root / ".agents/directives/read-budgets.json"
        if (not budget.is_file() or any(p.is_symlink() for p in
                (root, root / ".agents", root / ".agents/directives", budget))):
            raise ValueError("missing or linked budget definition")
        with budget.open("rb") as stream:
            raw = stream.read(65537)
        if len(raw) > 65536:
            raise ValueError("oversized read budget")
        data = json.loads(raw)
        if data["schema_version"] != 1 or set(data["scenarios"]) != {
            "startup", "code-edit", "source-maintenance", "plan-authoring", "release"
        }:
            raise ValueError("invalid scenarios")
        for name, scenario in data["scenarios"].items():
            files = scenario["files"]
            baseline, maximum = scenario["baseline_bytes"], scenario["max_bytes"]
            if (not isinstance(files, list) or not 1 <= len(files) <= 64
                    or len(set(files)) != len(files) or "AGENTS.md" not in files
                    or type(baseline) is not int or type(maximum) is not int
                    or not 0 < maximum <= baseline):
                raise ValueError("invalid budget bounds")
            required = {"AGENTS.md", ROUTER_TARGETS[1], ROUTER_TARGETS[7]}
            if name != "startup":
                required.update(ROUTER_TARGETS[i] for i in (0, 3, 4, 5, 6, 8))
                required.update(".agents/directives/references/" + item for item in
                                ("git-commits.md", "knowledge-and-preservation.md", "session-manifest.md"))
            phase_references = {
                "startup": (),
                "code-edit": ("verification.md", "plan-reconciliation.md"),
                "source-maintenance": ("documentation-verification.md", "plan-reconciliation.md"),
                "plan-authoring": ("documentation-verification.md", "planning-contract.md"),
                "release": ("verification.md", "plan-reconciliation.md", "git-branches.md",
                            "ci-and-candidates.md", "release-qualification.md", "stable-plan-gate.md",
                            "update-and-removal.md", "release-notes.md"),
            }
            required.update(".agents/directives/references/" + item for item in phase_references[name])
            if name == "release":
                required.add(ROUTER_TARGETS[2])
            if not required.issubset(files):
                raise ValueError("required phase reference omitted")
            total = 0
            for relative in files:
                if (not isinstance(relative, str) or "\\" in relative or ":" in relative
                        or any(part in ("", ".", "..") for part in relative.split("/"))
                        or not (relative == "AGENTS.md" or relative.startswith(".agents/directives/"))
                        or not relative.endswith(".md")):
                    raise ValueError("unsafe read path")
                path = root / relative
                current = root
                linked = root.is_symlink()
                for part in relative.split("/"):
                    current /= part
                    linked = linked or current.is_symlink()
                if not path.is_file() or linked:
                    raise ValueError("missing or linked directive")
                path.resolve().relative_to(root.resolve())
                total += path.stat().st_size
            metrics[name] = {"read_bytes": total, "baseline_bytes": baseline,
                             "max_bytes": maximum,
                             "reduction_percent": round((1 - total / baseline) * 100, 1)}
            if total > maximum:
                failures.append({"code": "phase-read-budget", "scenario": name, "actual": total})
    except (OSError, ValueError, TypeError, KeyError) as error:
        failures.append({"code": "invalid-read-budget", "reason": str(error)})
    return metrics, failures


def run() -> dict[str, object]:
    failures: list[dict[str, object]] = []
    source_agents = ROOT / "AGENTS.md"
    consumer_router = ROOT / "harness/template/AGENTS.md.jinja"
    source_bytes = sum(path.stat().st_size for path in SOURCE_FILES)
    metrics = {
        "source_agents_bytes": source_agents.stat().st_size,
        "source_directive_bytes": source_bytes,
        "source_reduction_percent": round((1 - source_bytes / SOURCE_BASELINE) * 100, 1),
        "consumer_router_bytes": consumer_router.stat().st_size,
        "consumer_reduction_percent": round(
            (1 - consumer_router.stat().st_size / CONSUMER_BASELINE) * 100, 1
        ),
    }
    if metrics["source_agents_bytes"] > 8 * 1024:
        failures.append({"code": "source-agents-size", "actual": metrics["source_agents_bytes"]})
    if source_bytes > SOURCE_BASELINE * 0.75:
        failures.append({"code": "source-directive-budget", "actual": source_bytes})
    read_sets, read_failures = check_read_budgets(ROOT)
    failures.extend(read_failures)
    reference_bytes = sum(path.stat().st_size for path in
                          (ROOT / ".agents/directives/references").rglob("*.md"))
    metrics.update(source_reference_bytes=reference_bytes,
                   source_total_bytes=source_bytes + reference_bytes,
                   reading_scenarios=read_sets,
                   measurement="UTF-8 file bytes; excludes plans, chat, tools and model reasoning")
    if metrics["consumer_router_bytes"] > CONSUMER_BASELINE * 0.5:
        failures.append({"code": "consumer-router-budget", "actual": metrics["consumer_router_bytes"]})

    agents_text = source_agents.read_text(encoding="utf-8")
    for target in ROUTER_TARGETS:
        if target not in agents_text or not (ROOT / target).is_file():
            failures.append({"code": "missing-source-route", "target": target})
    consumer_text = consumer_router.read_text(encoding="utf-8")
    for name in ("00-project-harness.md", "01-project-knowledge.md", "02-project-upgrade.md", "03-session-coordination.md"):
        if name not in consumer_text:
            failures.append({"code": "missing-consumer-route", "target": name})

    for source in CONSUMER_DIRECTIVES:
        projected = ROOT / "harness/template/.agents/directives" / source.name
        if not projected.is_file() or source.read_bytes() != projected.read_bytes():
            failures.append({"code": "directive-projection-drift", "target": source.name})

    source_duplicates = duplicate_findings(tuple(path for path in SOURCE_FILES if path.name != "00-editing-discipline.md"))
    consumer_duplicates = duplicate_findings(CONSUMER_DIRECTIVES + (ROOT / "harness/skills/verified-workflow/SKILL.md",))
    for scope, findings in (("source", source_duplicates), ("consumer", consumer_duplicates)):
        for finding in findings:
            failures.append({"code": "duplicate-normative-bullet", "scope": scope, **finding})

    verified = (ROOT / "harness/skills/verified-workflow/SKILL.md").read_text(encoding="utf-8")
    if ".agents/directives/00-project-harness.md" not in verified:
        failures.append({"code": "verified-workflow-route-missing"})
    for forbidden in ("Abort the continued task only", "Before a whole Goal or task becomes"):
        if forbidden in verified:
            failures.append({"code": "verified-workflow-common-rule-copy", "text": forbidden})

    renderer = (ROOT / "crates/hive-render/src/lib.rs").read_text(encoding="utf-8")
    function = renderer.split("fn render_agents_marker(", 1)[1].split("fn merge_shared_marker", 1)[0]
    for forbidden in ("Exact bad → good examples", "let marker = format!("):
        if forbidden in function:
            failures.append({"code": "renderer-agent-template-copy", "text": forbidden})

    return {"schema_version": 1, "metrics": metrics, "failure_count": len(failures), "failures": failures}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", choices=("json",), default="json")
    parser.parse_args()
    result = run()
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 1 if result["failure_count"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
