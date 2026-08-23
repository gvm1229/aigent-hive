#!/usr/bin/env python3
"""Build the deterministic 120-query vector feasibility corpus from bilingual source facts."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import yaml


PARAPHRASES = {
    "adversarial-judge": "Who challenges an implementation in a separate clean context before acceptance?",
    "agent-autonomous-continuation": "When may work stop while machine-owned steps still remain?",
    "agent-directive-ownership": "Where is each behavioral rule defined exactly once?",
    "artifact-boundaries": "How are the checkout, distributable bundle, and installed files kept apart?",
    "automated-user-handoff": "Which safe steps must finish before asking the maintainer to take over?",
    "automatic-dispatch-guard": "What prevents unattended task launch when quota evidence is weak?",
    "automatic-user-projection-refresh": "How do personal host instructions receive a new product projection?",
    "consumer-session-coordination": "How are concurrent editors stopped from claiming the same generated paths?",
    "crate-ownership": "Which Rust component is responsible for each product subsystem?",
    "dev-check-platform-path": "How does the developer check find platform tools without changing the caller environment?",
    "developer-binary-lifecycle": "How is a locally built executable distinguished from a published package?",
    "docs-wiki-architecture": "How does project documentation become a searchable bilingual source corpus?",
    "git-worktree-lifecycle": "When must an isolated checkout be removed after its branch work finishes?",
    "global-knowledge-bundle-transfer": "How can portable user knowledge move between machines without copying indexes?",
    "global-knowledge-rag": "Where are reusable personal facts searched across otherwise unrelated projects?",
    "global-onboarding": "What establishes a user's shared Hive environment before project setup?",
    "global-user-contexts": "How are personal preferences available outside a registered repository?",
    "graphify-0-10-adoption": "Which limited code relationship role is accepted for the external graph extractor?",
    "historical-project-base-coverage": "How does a direct upgrade authenticate every previously shipped project template?",
    "hive-preserving-uninstall": "What remains on disk when the executable integration is removed?",
    "host-external-integrations": "How are chat and workspace services connected without making them core runtime dependencies?",
    "host-neutral-continuation": "What keeps unfinished work moving across different agent hosts without an endless hook?",
    "hybrid-vector-search-0-10": "When may semantic nearest-neighbor retrieval join keyword and relationship search?",
    "install-wide-knowledge-capture": "Where can a reusable user decision be remembered even outside configured projects?",
    "installed-usage-guard": "Which installed mechanism decides whether autonomous work may consume more quota?",
    "interactive-binary-update": "How does a person approve and recover an executable replacement from the terminal?",
    "judge-verification": "What evidence is required before independent reviewers can authorize completion?",
    "knowledge-cross-project-access": "How can one repository intentionally retrieve another repository's allowed facts?",
    "knowledge-portability-scan": "How are useful Markdown claims discovered without letting a directory scan escape its root?",
    "knowledge-preservation": "What must happen to valid information before a guide is shortened or removed?",
}

ALTERNATE_PARAPHRASES = {
    "adversarial-judge": "Which review step looks for counterexamples but cannot approve its own findings?",
    "agent-autonomous-continuation": "Why is a failed test normally a reason to keep working instead of returning control?",
    "agent-directive-ownership": "How does Hive avoid copying the same instruction into several agent files?",
    "artifact-boundaries": "Which separation prevents development-only rules from reaching customer projects?",
    "automated-user-handoff": "What work should the agent complete before listing actions that require human authority?",
    "automatic-dispatch-guard": "Which check fails closed before background execution when usage is unknown?",
    "automatic-user-projection-refresh": "What replaces stale personal Skill files after an upgrade?",
    "consumer-session-coordination": "Which lease-like record protects generated files from simultaneous sessions?",
    "crate-ownership": "Where should a change go when it belongs to rendering rather than update recovery?",
    "dev-check-platform-path": "What keeps a development command from permanently rewriting PATH?",
}


def read_fact(path: Path) -> tuple[dict[str, object], str]:
    text = path.read_text(encoding="utf-8")
    _, frontmatter, body = text.split("---", 2)
    value = yaml.safe_load(frontmatter)
    if not isinstance(value, dict):
        raise ValueError(f"invalid fact frontmatter: {path}")
    return value, body.strip()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--facts", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    pairs = []
    for english_path in sorted((args.facts / "en").glob("*.md"))[:30]:
        english, body = read_fact(english_path)
        pair_id = str(english["pair_id"])
        korean, _ = read_fact(args.facts / "ko" / english_path.name)
        pairs.append((pair_id, english, korean, body))
    if len(pairs) != 30:
        raise SystemExit("vector corpus requires at least 30 bilingual facts")

    documents = [
        {
            "id": pair_id,
            "text": f"{pair_id}\n{english['title']}\n{body}",
        }
        for pair_id, english, _, body in pairs
    ]
    queries = []
    for pair_id, _, _, _ in pairs:
        queries.append({"id": f"exact-{pair_id}", "kind": "exact", "query": pair_id, "expected": [pair_id]})
    for pair_id, _, _, _ in pairs:
        queries.append({"id": f"paraphrase-primary-{pair_id}", "kind": "paraphrase", "query": PARAPHRASES[pair_id], "expected": [pair_id]})
    for pair_id, _, _, _ in pairs[:10]:
        queries.append({"id": f"paraphrase-alternate-{pair_id}", "kind": "paraphrase", "query": ALTERNATE_PARAPHRASES[pair_id], "expected": [pair_id]})
    for pair_id, _, korean, _ in pairs[:20]:
        queries.append({"id": f"cross-language-{pair_id}", "kind": "cross-language", "query": str(korean["summary"]), "expected": [pair_id]})
    for pair_id, english, _, _ in pairs:
        links = english.get("links")
        relation = str(links[0]) if isinstance(links, list) and links else "the Hive product"
        queries.append({"id": f"relation-{pair_id}", "kind": "relation", "query": f"How does {english['title']} relate to {relation}?", "expected": [pair_id]})
    counts = {kind: sum(query["kind"] == kind for query in queries) for kind in ("exact", "paraphrase", "cross-language", "relation")}
    if counts != {"exact": 30, "paraphrase": 40, "cross-language": 20, "relation": 30}:
        raise SystemExit(f"invalid vector corpus counts: {counts}")
    payload = {"schema_version": 1, "documents": documents, "queries": queries}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
