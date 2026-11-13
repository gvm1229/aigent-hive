# 02. Architecture Directive

Runtime, orchestration, artifacts, Skills and data. Filesystem/credentials: `05-security-safety.md`.

## Artifact boundary

Keep source (code/tests/directives/docs), release bundles (reproducible binaries/versioned
projections), and installed harnesses (generated files in independent consumer projects) separate.
No consumer state in source, source directives in releases, or user data in releases/plugin caches.
`harness/` alone owns consumer templates/Skills; host files are thin projections.

## Runtime and orchestration

- Use subscription-authenticated hosts and official provider CLI extensions only. No provider APIs,
  credential requests or model/subagent process launch.
- No provider CLI key injection/screenshots, including tmux/psmux send-keys/capture-pane or cmux.
  Claude stays interactive; no headless Claude.
- The active vector plan permits a separately consented, non-generative local embedding indexer,
  not prompts, text generation, or agent execution.
- Hosts own model calls/processes. Hive owns deterministic setup, canonical state, projection,
  indexing, validation, migration, scheduling, leases, receipts, cancellation, recovery, Judge
  contracts, and rollback.
- Implement Hive-native iterative planning, bounded retry, logical scheduling, team coordination, and
  multi-goal execution with provider-neutral state machines, Skills, and declarative envelopes.
- New orchestration stays off until hosts prove envelope consumption, typed receipts, cancellation
  and safe reclaim. State unsupported/unverified capabilities; uncertain launch stays `dispatch-uncertain`.
- Exactly-once dispatch claims require a qualified receipt and idempotency contract.
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

## CLI display

- Human CLI defaults use one shared ActionResult renderer in the configured language: result,
  before/after, recovery and next step. No raw JSON. JSON is opt-in (`--output json`); preserve exact
  machine contracts and explicit worker/hook protocols. Match human help/examples.

## Skill boundary

- Use narrow descriptions and host-native discovery; do not add a duplicate prompt-classifier hook.
- Keep execution Skills declarative: validate state and prepare envelopes for official host
  extensions only. No provider calls, process launch, CLI key injection, or screen capture.
- Product Skills: `harness/skills/<name>/`. Source exceptions: `.agents/skills/update-summary/`
  and `.agents/skills/draft-devlog/`; neither ships. No general source Skill inventory.
- Reuse a consumer workflow in source only after scope, consent, safety, and conformance review.
  Never copy installed project state or weaken source-root, usage, or mutation boundaries.
- Apply a Skill rename through the complete current inventory and transitive rename ledger in one
  reviewed migration. Historical release bases remain immutable.
- Load only relevant approved Skills; none unrelated for simple questions.

## Canonical data

- Markdown owns knowledge, roles, plans, run status, and evidence.
- Tracked typed YAML/TOML owns setup, approval, and suppression authority.
- SQLite is disposable, rebuilt without models/network; facts cannot live there alone.
  Canonical FTS follows this rule. Optional derived vectors may use the approved local embedding
  indexer; its absence must not affect FTS or canonical knowledge.
- Exact bilingual source fact pairs: `docs/facts/en/` and `docs/facts/ko/`.
- Never use OMX Wiki or consumer `.hive/knowledge/` as source knowledge.
