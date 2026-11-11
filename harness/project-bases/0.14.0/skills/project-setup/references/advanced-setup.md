# Explicit advanced setup

Read only for an explicitly requested advanced capability, optional Skill/hook, or supplied host
evidence. The CLI owns questions and validates these constraints. Do not start another questionnaire,
construct capability evidence from guesses, or launch a Judge or subagent for ordinary setup.

4. Resolve the v0.9 owner and capabilities.
   - Default a new run to the active host's verified native capabilities without asking an owner question.
   - Do not select OMX/OMC for a new run. Existing external owner pins are historical provenance;
     their presence never authorizes invocation or changes the host-native default.
   - Preserve any owner already pinned by an existing run, including a 0.8.x OMX/OMC owner. Reconfiguration never migrates or replaces that owner implicitly.
   - Normalize host-native capability support to `supported`, `best-effort`, `unsupported`, or `unverified`. A required `unsupported` or `unverified` capability stops before writes with `host_capability_unsupported`.
   - Retain `available`, `absent`, `incompatible`, or `unknown` external detection only for an explicitly selected compatibility layer or validation of legacy 0.8.x state.
   - Compute `evidence_digest` as `sha256(RFC 8785 JCS(normalized capability-resolution object excluding only evidence_digest))`. This binds host, version, surface, detection, external runtime, resolved owner, capability claims, and evidence.
   - Positive evidence may come only from active-host Skill/plugin capability metadata or a public executable path with side-effect-free `--version`.
   - An explicitly selected external layer requires positive compatibility evidence. Missing, incompatible, or unknown evidence is unsupported and never triggers an automatic owner fallback.
   - Never install, update, configure, or invoke OMX/OMC.
   - Never infer state by reading `.omx/`, `.omc/`, plugin caches, session state, or host-global configuration.
   - Install Hive-native iterative, team, multi-goal, or scheduler capabilities only when the signed release implements them, the selected host passes the required capability checks, the feature is activated, and every optional Skill has explicit approval.
   - Never substitute an OMX/OMC runtime, namespace, command, or state tree for a missing Hive-native capability.
5. Present optional Skills individually.
   - Show name, purpose, source, pinned revision, content digest, requested tools, and overlap.
   - Require explicit approval for every requested capability and Skill. An empty approval list is valid.
   - Sort capability IDs and bind consent version, displayed provenance, content digest, requested/approved capabilities, and UTC-seconds approval time with `sha256(RFC 8785 JCS(payload))`.
   - Recompute the digest before staging and never repair a mismatch automatically.
6. Offer optional data-integrity hooks only on a supported host-native hook surface.
   - Do not ask, render, register, or execute a hook when the host-native hook surface is `unsupported` or `unverified`, or when an explicit external compatibility owner is active.
   - Show each exact capability, event, `.hive/hooks/<capability>` path, `hive hook --capability <capability> --event <event> --capabilities .hive/runtime/current-capability-resolution.json --output json` command, and content digest.
   - Require explicit approval per hook capability. Declining every hook is fully supported and must create no hook approval artifact or command.
   - Limit hooks to `protect-hive-owned-state`, `update-integrity-guard`, `derived-state-invalidation`, `checkpoint-reminder`, `continue-active-run`, and `validate-korean-output`.
   - Require the host adapter to refresh `.hive/runtime/current-capability-resolution.json` immediately before each non-Stop invocation. Setup never creates or tracks this ephemeral file; missing, stale, malformed, unsupported, or unverified host-native evidence leaves the hook inert before hook input is read.
   - Hook adapters pass a versioned JSON object on stdin. It contains `schema_version` and `event`, plus only the typed fields needed by the approved capability: `tool`/`operation`/`path`, `action` with dry-run/backup/staging gates, canonical `path`, or `status_path`/`checkpoint_present`.
   - Hooks never classify prompt-submission events, rewrite prompts, activate Skills, ingest memory, spawn subagents, orchestrate work, or decide continuation.
   - A `Stop` hook always returns a neutral allow result without a block decision, continue instruction, or re-invocation prompt.

## Project Skill choices

- Show the complete daily-work list before apply, one Skill per entry with its purpose.
  Exclude user-setup, project-setup, custom-subagent-create, knowledge-transfer and
  project-transition from the default; user-setup remains user-scope only.
- Offer the full supported project catalog for customization, including the four opt-in
  specialized Skills. Never invent a small individual selection on the user's behalf.
- If the user customizes Skills during expedited setup, reuse the validated inherited language,
  Wiki and persona as custom answers; ask only about the requested selection.
- Show dependency closure before apply. Removing a required Skill also requires removing its
  dependent selections; never silently re-add a Skill the user explicitly excluded.
- A disabled Wiki excludes the five daily-work knowledge Skills and explains the 18-Skill result.
- Show selected Skills and their verified plugin or local delivery. Physical folder count is not
  availability. Selected Skills may route relevant natural-language requests; they do not grant
  file mutation, publication, external delivery, model launches or hook consent.

## Fixed product decisions

Do not ask about these:

- Local-only operation on an authenticated subscription host
- No provider API calls or provider API keys
- Markdown canonical knowledge/role/run data plus typed tracked YAML/TOML configuration and consent, with a rebuildable SQLite index
- Git tracking for non-confidential canonical files
- SQLite, WAL/SHM, generated backup, and runtime cache exclusion
- Seven-day maximum backup retention
- Hive-native iterative, team, multi-goal, and logical-scheduler capabilities remain unavailable until their signed release, host qualification, and activation gates pass
- No provider session engine, model runtime, direct model/subagent process launcher, or OMX/OMC runtime dependency
- Orchestration owner selection: new runs use verified host-native capabilities; existing external owner pins remain unchanged and never authorize new invocation
- Owner detection evidence sources: active-host capability metadata and side-effect-free public `--version` only

## Safety Invariants

- Never call a model-provider API or request an API key.
- Never write a consumer harness into the Hive source workspace.
- Never activate an optional Skill without explicit approval.
- Never activate a Skill whose consent payload or digest fails validation.
- Never claim persistent completion, subagents, or usage enforcement when the selected host does not expose the required capability.
- Never silently switch from a host-native, explicitly selected external, or legacy-pinned owner to another orchestration layer.
- Never offer or install data-integrity hooks without a supported host-native hook surface and exact user consent, or while an explicit external compatibility owner is active.
- Never treat `best-effort`, `unsupported`, or `unverified` as supported.
- Never allow a fallback `Stop` hook to continue or block a session.
- Never use Copier directly against the live consumer tree; Copier is an authoring and CI surface.
