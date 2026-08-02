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
- The active host owns model calls and model/subagent processes and consumes Hive-produced declarative execution envelopes.
- Hive owns setup, deterministic projection, Markdown data contracts, SQLite indexing, validation, migration, update safety, iterative execution judgment, logical scheduling, leases, receipts, cancellation, team coordination, and multi-goal state.
- Hive orchestration must remain a local deterministic control plane, not a provider client, model runtime, process launcher, or opaque session daemon.

## Orchestration Boundary

- Implement Hive-native iterative planning, bounded retry, logical scheduling, team coordination, and multi-goal execution as provider-neutral state machines and Skills.
- Keep new orchestration capability default-off until the relevant host proves envelope consumption, typed receipts, cancellation, and safe reclaim behavior. An unsupported capability produces a truthful unsupported or `dispatch-uncertain` result, never an automatic backend switch.
- Do not select, invoke, install, or configure OMX/OMC for a new workflow. Preserve legacy `.omx/.omc` bytes and pinned owner metadata as foreign read-only provenance.
- Migrate a legacy external-owner run only through an explicit validated command that creates a new Hive-native run identity and leaves the original owner and bytes unchanged.
- Treat ambient or selected session pointers as selectors only, never mutation authority. Bind every state mutation to the exact target, expected event head, control epoch, request digest, and authenticated single-action authority.
- Keep status, cancellation, recovery, and usage-guard control reachable by exact ID without scheduler locks or selected-session pointers. Normal cancellation commits through the canonical event head; corrupt-head emergency cancellation uses a separately authenticated bounded recovery path.
- An optional host lifecycle hook requires a supported exact event, capability preview, scoped authority, exact run revision, one-time action binding, and explicit user approval. Never infer authority from a pointer, transcript, current directory, or hook session selection.
- Never claim exactly-once dispatch without a qualified host receipt and idempotency contract. When launch status cannot be proven, stop at `dispatch-uncertain` and require proof-gated recovery.

## Skill Routing Boundary

- Use Skill descriptions and compact routing directives for semantic task-to-Skill selection; do not build a duplicate prompt-classifier hook.
- Let host-native discovery and narrow Hive Skill descriptions route work into Hive-owned iterative, team, multi-goal, planning, verification, setup, knowledge, migration, and update workflows.
- Do not route new work to OMX/OMC Skills. A legacy external artifact may be inspected only through an explicit migration or recovery contract that preserves foreign bytes.
- Keep Hive execution Skills declarative: they may reduce canonical state, issue bounded leases, validate receipts, and prepare host envelopes, but never call a model-provider API or launch a model/subagent process.
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
- Source Wiki facts are canonical tracked Markdown under `docs/facts/en/` and
  `docs/facts/ko/`. Keep one primary fact per exact bilingual pair and connect it to the
  human-readable `docs/` graph.
- Do not use `omx_wiki/`, `.omx/wiki/`, or consumer `.hive/knowledge/` for source knowledge.
  Keep the source Wiki SQLite projection ignored under `.agents/work/source-wiki/`.

## Source Layout

- Root `.agents/` governs Hive development and is never a shipping source.
- `harness/` is the only canonical source for consumer templates, portable skills, and host projections.
- Host-specific files must be thin projections from provider-neutral contracts.
- Do not create empty crates or adapters that imply unsupported functionality.
