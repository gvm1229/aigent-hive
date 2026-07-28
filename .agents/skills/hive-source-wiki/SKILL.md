---
name: hive-source-wiki
description: Query, lint, index, capture, or maintain Aigent Hive's provider-neutral bilingual source Wiki. Route explicit source Wiki work here and load it automatically at the completion gate when a material source task produced durable task facts; never intercept ordinary simple questions or consumer-project knowledge work.
---

# Hive Source Wiki

Operate only on the Aigent Hive source workspace and its tracked bilingual source knowledge.

## Boundaries

- Require a source root identified by `hive-source.json`.
- Treat `llm-wiki/en/` and `llm-wiki/ko/` as the only canonical source Wiki.
- Treat `.agents/work/source-wiki/index.sqlite3` as ignored, disposable, and rebuildable.
- Treat `.agents/work/source-wiki/.index.lock` as an ignored, persistent, noncanonical
  coordination marker. Preserve the regular file; rebuild serialization uses its OS advisory
  lock rather than file creation or deletion as ownership.
- Keep SQLite creation off ambient target paths: in-memory construction, serialize/deserialize
  verification, then recoverable two-phase CAS publication through the pinned source-root
  capability.
- Let `lint` and `query` wait for the persistent marker's shared reader lock before reading the
  live index. Keep the lock through bounded read and in-memory verification so readers never
  observe an in-flight claim gap.
- Treat a missing live index plus an exact Hive-owned orphan claim after a crash as disposable
  derived state. Only the next explicit `index` rebuild may reconstruct from canonical Markdown
  and clean exact regular Hive-owned claim or temporary paths. `lint` and `query` remain
  fail-closed and never repair derived state.
- Keep every English and Korean page as an exact pair with the same pair identity and reviewed source locators.
- Reuse provider-neutral consumer knowledge safety workflows and core primitives, not an installed consumer layout, runtime state, or knowledge.
- Never use or mutate `.omx/`, `.omc/`, `omx_wiki/`, or `.hive/`.
- Never promote source Wiki content into the consumer product without a separate explicit review.

## Read-only workflow

Choose the smallest command that satisfies the explicit request:

```text
hive source-wiki lint --target <source-root> --output json
hive source-wiki index --target <source-root> --output json
hive source-wiki query --target <source-root> --language en|ko \
  (--text <query>|--tag <tag>) [--limit <1..100>] --output json
```

- Run `lint` before relying on pair, metadata, link, source locator, secret-safety, or current
  index claims. It may wait for a writer and reports missing, stale, or corrupt derived state
  without repair.
- Run `index` only to rebuild the disposable SQLite projection.
- Run `query` only for explicit source Wiki lookup intent. Missing, stale, corrupt, or
  crash-interrupted derived state fails closed until an explicit rebuild.
- Keep ordinary simple questions on the simple-question path without loading this Skill.

## Capture and maintenance workflow

1. Require either explicit source selection or the source completion gate for facts derived only
   from the current authorized task and reviewed local artifacts.
2. Reject secret, credential-adjacent, private account, captured session, or unreviewed external material.
3. Edit tracked files only inside `llm-wiki/en/` and `llm-wiki/ko/`.
4. Update both language files in the pair together.
5. Preserve the pair identity, reciprocal counterpart path, reviewed source locators, and matching source locator digests.
6. Prefer current truth over additive correction history.
7. Run `lint` and repair canonical findings. Before the first rebuild, only a standalone
   `stale-index` finding may be expected.
8. Rebuild with `index`, rerun `lint`, then confirm both language paths with `query`.
9. Review the tracked diff and keep the SQLite index and persistent lock marker untracked.

## Agent-Reviewed Task-Fact Autocapture

- At the completion gate, capture a material task fact without another prompt when the current
  authorized task created or substantially revised reusable Hive source knowledge.
- Preserve the outcome, tool or external project, creation or acceptance criteria, and a bounded
  originating request summary. Preserve exact request text only after explicit user retention
  intent and safety review.
- Record external-artifact facts through a safe tracked source handoff; do not import consumer
  project files or installed knowledge into `llm-wiki/`.
- Update one current-truth bilingual topic pair and rebuild the index. Identical input is a no-op.
- Do not capture an editless/simple question, raw transcript, complete conversation, hook payload,
  tool output, hidden prompt, cache, database, or runtime state.

Do not ingest an external source without selection and review. Do not mutate consumer knowledge,
install state, or orchestration state.
