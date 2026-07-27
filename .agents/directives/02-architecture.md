# 02. Architecture Directive

This directive protects Aigent Hive's product boundaries.

## Artifact Classes

Maintain three distinct artifact classes:

1. **Hive source workspace** — this repository's Rust source, templates, schemas, tests, directives, and documentation.
2. **Release bundle** — reproducible binaries plus versioned template/projection data and verification metadata.
3. **Installed harness** — files generated inside an independent consumer project.

Never import installed consumer state back into source, copy source-development directives into a consumer harness, or store user data in a plugin cache or release directory.

## Runtime Boundary

- Hive runs locally on top of an already authenticated subscription host.
- Hive must not call Anthropic, OpenAI, Google, or other model APIs.
- Hive must not request, read, store, or forward provider API keys.
- The active host or external orchestration layer owns model calls, subagent processes, session continuation, and retries.
- Hive owns setup, deterministic projection, Markdown data contracts, SQLite indexing, validation, migration, and update safety.

## Orchestration Boundary

- Do not implement Hive equivalents of plan, Ralph, team, swarm, or provider session runtimes.
- Prefer a compatible OMX capability on Codex and OMC capability on Claude whenever it is available in the active host. Otherwise use only what the host natively supports.
- Treat OMX and OMC as replaceable compatibility dependencies with planned retirement, not as Hive authorities. Do not bind canonical data, paths, schemas, Skill identity, or durable knowledge to their namespaces. Retire an adapter only after a provider-neutral or host-native owner covers its required capability.
- Do not ask the user to choose a pure-Hive orchestration mode. Resolve one owner per run from the active host capability surface and pin it for that run.
- Preserve OMX/OMC namespaces and let the resolved layer own orchestration. Do not install Hive lifecycle hooks when OMX or OMC is detected.
- If neither OMX nor OMC is detected, any Hive fallback hook requires an explicit capability preview and user approval before installation.
- If a required capability is unavailable, report it as unsupported. Do not silently emulate it or switch backends.

## Skill Routing Boundary

- Use Skill descriptions and compact routing directives for semantic task-to-Skill selection; do not build a duplicate prompt-classifier hook.
- Prefer an existing OMX/OMC Skill over a Hive duplicate.
- Keep Hive Skills focused on Hive-owned setup, prompt refinement, canonical knowledge, role/run handoff, judge packaging, migration, and update contracts.
- Permit bidirectional reuse only for Hive-owned Skill source after source/consumer scope, safety, consent, and conformance review. Never treat an installed consumer copy or consumer runtime state as source material.
- Keep a shared Skill canonical under `harness/skills/<name>/` and project an exact source copy under `.agents/skills/<name>/`. Keep a source-only Skill under `.agents/skills/<name>/` until an explicit product-relevance review promotes it to `harness/skills/`.
- A consumer Skill reused in source must not require consumer `.hive/` state, mutate an installed harness, weaken source-root refusal, or bypass the source usage guard. Adapt the provider-neutral workflow or core primitive instead of copying consumer state assumptions.
- Load only the smallest approved Skill set needed for the task. The simple-question path loads no unrelated project Skill or memory.

## Canonical Data

- Knowledge, role identity, run state, plans, status, and evidence manifests are canonical Markdown.
- Setup answers, typed configuration, approval ledgers, and suppression fingerprints are canonical tracked YAML/TOML.
- Raw source objects may retain their original non-Markdown format when small, non-confidential, and Git-suitable.
- SQLite is a derived local index and may be deleted at any time.
- No durable fact may exist only in SQLite.
- A clean checkout containing the canonical tracked Markdown/YAML/TOML and source objects must be sufficient to rebuild the index without a model call or network request.
- Source LLM Wiki content, when present, is canonical tracked Markdown under `llm-wiki/en/` and `llm-wiki/ko/`. Do not use `omx_wiki/`, `.omx/wiki/`, or consumer `.hive/knowledge/` for source knowledge. Keep any source Wiki SQLite projection ignored under `.agents/work/source-wiki/`.

## Source Layout

- Root `.agents/` governs Hive development and is never a shipping source.
- `harness/` is the only canonical source for consumer templates, portable skills, and host projections.
- Host-specific files must be thin projections from provider-neutral contracts.
- Do not create empty crates or adapters that imply unsupported functionality.
