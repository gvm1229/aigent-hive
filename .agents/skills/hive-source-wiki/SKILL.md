---
name: hive-source-wiki
description: Query, lint, index, capture, or maintain Aigent Hive's provider-neutral bilingual source Wiki. Use it for the source workspace's automatic pre-work knowledge lookup and the material-task completion gate; never route the source root through consumer knowledge retrieval or intercept consumer-project knowledge work.
---

# Hive Source Wiki

Operate only on the Aigent Hive source workspace and its tracked `docs/` Wiki.

## Boundaries

- Require a source root identified by `hive-source.json`.
- Treat `docs/facts/en/` and `docs/facts/ko/` as the canonical atomic fact corpus inside
  the wider `docs/` Wiki.
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
- Keep every English and Korean page as an exact pair with the same pair identity and reviewed
  source locators.
- Keep one primary fact per pair. Split unrelated facts into linked pairs instead of adding
  sections to an existing fact.
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

- For the automatic pre-work lookup required by the source behavior directive, detect
  `hive-source.json` first and run the bounded `source-wiki query` command directly. Never call
  consumer `hive knowledge retrieve` with the source root, and never count its expected source
  guard conflict as a completed lookup.
- Run `lint` before relying on pair, metadata, link, source locator, secret-safety, or current
  index claims. It may wait for a writer and reports missing, stale, or corrupt derived state
  without repair.
- Run `index` only to rebuild the disposable SQLite projection.
- Run `query` for explicit source Wiki lookup intent or the single automatic source pre-work
  lookup. Missing, stale, corrupt, or crash-interrupted derived state fails closed until an
  explicit rebuild.
- Keep ordinary simple questions on the simple-question path without loading this Skill.

## Capture and maintenance workflow

1. Require either explicit source selection or the source completion gate for facts derived only
   from the current authorized task and reviewed local artifacts.
2. Reject secret, credential-adjacent, private account, captured session, or unreviewed external material.
3. Edit fact pairs only inside `docs/facts/en/` and `docs/facts/ko/`. Update human-facing topic
   documents, maps, or the index separately when the fact changes navigation or explanation.
4. Update both language files in the pair together.
5. Preserve one primary fact, pair identity, reciprocal counterpart path, reviewed source
   locators, and matching source locator digests.
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
  project files or installed knowledge into `docs/facts/`.
- Update one current-truth bilingual topic pair and rebuild the index. Identical input is a no-op.
- Do not capture an editless/simple question, raw transcript, complete conversation, hook payload,
  tool output, hidden prompt, cache, database, or runtime state.

Do not ingest an external source without selection and review. Do not mutate consumer knowledge,
install state, or orchestration state.
