# 02. Architecture Directive

Owns runtime, orchestration, artifact, Skill, and canonical-data boundaries. Filesystem mutation
and credentials: `05-security-safety.md`.

## Artifact boundary

Separate:

1. Hive source workspace: source, tests, directives, and docs
2. Release bundle: reproducible binaries and versioned projections
3. Installed harness: generated files in an independent consumer project

Never import consumer state into source, ship source directives, or store user data in a release
or plugin cache. `harness/` is the only canonical consumer template and
Skill source; host-specific files remain thin projections.

## Runtime and orchestration

- Hive uses authenticated subscription hosts and only officially documented provider CLI extension
  points. Never call provider APIs, request credentials, or launch model/subagent processes.
- Never inject keystrokes into or capture screens of provider CLIs, including tmux/psmux
  send-keys/capture-pane and cmux. Claude stays interactive; no headless Claude.
- The active vector plan permits a separately consented, non-generative local embedding indexer,
  not prompts, text generation, or agent execution.
- Hosts own model calls/processes. Hive owns deterministic setup, canonical state, projection,
  indexing, validation, migration, scheduling, leases, receipts, cancellation, recovery, Judge
  contracts, and rollback.
- Implement Hive-native iterative planning, bounded retry, logical scheduling, team coordination, and
  multi-goal execution with provider-neutral state machines, Skills, and declarative envelopes.
- Keep new orchestration default-off until the host proves envelope consumption,
  typed receipts, cancellation, and safe reclaim. Report unsupported or unverified capability
  truthfully; uncertain launch remains `dispatch-uncertain`.
- Never claim exactly-once dispatch without a qualified receipt and idempotency contract.
- Bind mutation to exact target, event head, control epoch, request digest, and authenticated
  single-action authority. Ambient session pointers are selectors, never authority.
- Keep status, cancellation, recovery, and usage control addressable by exact ID without scheduler
  locks. Emergency cancellation uses its separately authenticated bounded recovery path.
- Optional host hooks need supported exact events, capability preview, scoped authority, exact
  revision, one-time binding, and explicit approval. A hook never derives authority from transcript,
  current directory, or ambient pointer.

## External-owner compatibility

- Do not select, invoke, install, or configure OMX/OMC for a new workflow.
- Preserve legacy `.omx/.omc` bytes and pinned owner metadata as foreign read-only provenance.
- Migrate legacy work only through an explicit validated action that creates a new Hive-native run
  identity and leaves the old owner and bytes unchanged.
- Never switch an existing run owner or silently substitute another runtime.

## Skill boundary

- Use narrow descriptions and host-native discovery; do not add a duplicate prompt-classifier hook.
- Keep execution Skills declarative: validate state and prepare envelopes for official host
  extensions only. No provider calls, process launch, CLI key injection, or screen capture.
- Keep reusable product Skills under `harness/skills/<name>/`. Only source-project exceptions: `.agents/skills/update-summary/` and `.agents/skills/draft-devlog/`; both are
  nonshipping, with no general source Skill inventory.
- Reuse a consumer workflow in source only after scope, consent, safety, and conformance review.
  Never copy installed project state or weaken source-root, usage, or mutation boundaries.
- Apply a Skill rename through the complete current inventory and transitive rename ledger in one
  reviewed migration. Historical release bases remain immutable.
- Load the smallest relevant approved Skill set. Simple questions load no unrelated Skill.

## Canonical data

- Markdown owns knowledge, roles, plans, run status, and evidence.
- Tracked typed YAML/TOML owns setup, approval, and suppression authority.
- SQLite is disposable and rebuildable without model or network access; no fact may exist only in
  SQLite.
  Canonical FTS follows this rule. Optional derived vectors may use the approved local embedding
  indexer; its absence must not affect FTS or canonical knowledge.
- Source facts live as exact bilingual pairs under `docs/facts/en/` and `docs/facts/ko/`.
- Never use OMX Wiki or consumer `.hive/knowledge/` as source knowledge.
