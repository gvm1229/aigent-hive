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
