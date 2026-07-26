---
name: setup-harness
description: Inspect a project and configure its local Aigent Hive harness through the signed Hive CLI. Use when a user asks to set up, initialize, install, reconfigure, or regenerate Aigent Hive; when creating AGENTS.md and scoped directives from a project profile; or when selecting a host, persistent roles, memory policy, usage guard, manually approved optional Skills, and eligible fallback data-integrity hooks.
---

# Setup Harness

Configure a consumer project without copying Hive source-development instructions or modifying third-party runtime state.

## Workflow

1. Locate the project root and inspect it read-only.
   - Read existing `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.gitignore`, project manifests, and `git status`.
   - Stop if `hive-source.json` is present.
   - Do not read secrets, provider credentials, `.omx/`, `.omc/`, or host-global configuration.
2. Check the signed Hive CLI.
   - Run `hive setup --help`.
   - If the command is unavailable, report that setup is unsupported in the installed release. Do not reproduce the renderer manually.
3. Ask one setup question at a time.
   - Follow the required sequence under `Setup Question Sequence` below.
   - Infer repository facts before asking the user.
   - Do not infer preference, risk tolerance, host choice, or optional Skill approval.
4. Resolve compatibility.
   - Do not ask the user to choose an orchestration owner.
   - On Codex, prefer compatible OMX. On Claude Code, prefer compatible OMC. Otherwise resolve host-native capability.
   - Normalize the read-only result to `available`, `absent`, `incompatible`, or `unknown`.
   - Compute `evidence_digest` as `sha256(RFC 8785 JCS(normalized capability-resolution object excluding only evidence_digest))`. This binds host, version, surface, detection, external runtime, resolved owner, capability claims, and evidence.
   - Positive evidence may come only from active-host Skill/plugin capability metadata or a public executable path with side-effect-free `--version`.
   - Conclusive `absent` requires both host-catalog absence and public executable absence. A missing probe surface is `unknown`, not `absent`.
   - Never install, update, configure, or invoke OMX/OMC.
   - Never infer state by reading `.omx/`, `.omc/`, plugin caches, session state, or host-global configuration.
   - Do not install Hive copies of plan, Ralph, team, persistent execution loops, or semantic routing hooks.
5. Present optional Skills individually.
   - Show name, purpose, source, pinned revision, content digest, requested tools, and overlap.
   - Require explicit approval for every requested capability and Skill. An empty approval list is valid.
   - Sort capability IDs and bind consent version, displayed provenance, content digest, requested/approved capabilities, and UTC-seconds approval time with `sha256(RFC 8785 JCS(payload))`.
   - Recompute the digest before staging and never repair a mismatch automatically.
6. Offer fallback hooks only after conclusive absence.
   - Do not ask, render, register, or execute a fallback hook when detection is `available`, `incompatible`, or `unknown`.
   - For `absent`, show each exact capability, event, `.hive/hooks/<capability>` path, `hive hook --capability <capability> --event <event> --capabilities .hive/runtime/current-capability-resolution.json --output json` command, and content digest.
   - Require explicit approval per hook capability. Declining every hook is fully supported and must create no hook approval artifact or command.
   - Limit hooks to `protect-hive-owned-state`, `update-integrity-guard`, `derived-state-invalidation`, and `checkpoint-reminder`.
   - Require the host adapter to refresh `.hive/runtime/current-capability-resolution.json` immediately before each non-Stop invocation. Setup never creates or tracks this ephemeral file; missing, stale, malformed, or non-absent evidence leaves the hook inert before hook input is read.
   - Hook adapters pass a versioned JSON object on stdin. It contains `schema_version` and `event`, plus only the typed fields needed by the approved capability: `tool`/`operation`/`path`, `action` with dry-run/backup/staging gates, canonical `path`, or `status_path`/`checkpoint_present`.
   - Hooks never classify prompt-submission events, rewrite prompts, activate Skills, ingest memory, spawn subagents, orchestrate work, or decide continuation.
   - A `Stop` hook always returns a neutral allow result without a block decision, continue instruction, or re-invocation prompt.
7. Run a dry run and validate the write set.
   - Require the CLI to render into staging first.
   - Reject writes outside Hive-owned paths and the exact Hive marker in shared files.
   - Show conflicts and unsupported host capabilities before mutation.
8. Apply setup.
   - Preserve non-Hive text and third-party marker blocks byte-for-byte.
   - Materialize each approved role seed into `.hive/team/roles/<role-id>.md` in staging.
   - On reconfigure, preserve existing assignment, handoff, and Markdown body; require explicit approval for definition drift.
   - Do not commit or push unless the user explicitly requested the Git operation.
9. Verify.
   - Run `hive setup --target <project-root> --answers <setup-answers.yml> --capabilities <capability-resolution.json> --validate --output json` with the same validated inputs used for the dry run and apply.
   - Confirm role seeds have schema-valid canonical role documents, Markdown canonical files exist, SQLite files are ignored, setup answers are tracked, and the selected host can discover the generated entrypoint.
   - Report generated paths, skipped optional components, detected limitations, and exact recovery steps.

## Setup Question Sequence

Ask only questions whose answers cannot be established from the repository.

## Required order

1. **Project identity**
   - Confirm the detected project name and repository root.
2. **Domain profile**
   - Offer only profiles present in the signed release.
   - Explain that `custom` installs the base contract without guessing domain rules.
3. **Primary host**
   - `codex`, `claude`, or `antigravity`.
4. **Persistent roles**
   - Ask which stable team roles the project needs.
   - Record `role_id`, display name, responsibilities, non-responsibilities, context paths, allowed capabilities, write scope, and verification duties.
   - Store role identity and handoff, not a permanent process or model session.
5. **Knowledge scope**
   - Confirm which project files may be ingested into Raw/Wiki.
   - Record explicit project-relative include and exclude paths/globs.
   - Exclude credentials, private keys, tokens, and unrelated repositories.
6. **Root knowledge promotion**
   - Ask which of `fact`, `preference`, and `workflow` may be explicitly promoted.
   - Ask which categories are confidential and must never leave the project.
   - Record a stable project identity plus the SHA-256 binding of the selected user store root.
   - An empty promotion category list keeps all knowledge project-local.
7. **Usage guard**
   - Default stop threshold: 10% remaining.
   - Explain that the threshold is enforceable only when a fresh local subscription usage sensor exists.
   - Without a trustworthy sensor, automatic continuation must fail closed.
8. **Judge policy**
   - Normal risk: one independent judge when requested.
   - Elevated risk: two of three independent judges.
   - Critical risk: three of three plus human approval.
9. **Optional Skills**
   - Present each candidate separately with provenance and overlap.
   - Present requested filesystem, shell, network, subagent, and external-app capabilities.
   - Require a direct approval or rejection for each capability and candidate.
   - Sort capability IDs and record `sha256(RFC 8785 JCS(payload))` over consent version, exact displayed name/source/revision/content digest, requested/approved capabilities, and UTC-seconds approval time.
10. **Optional fallback hooks**
   - Ask only when the automatic external capability result is conclusively `absent`.
   - Do not ask when the result is `available`, `incompatible`, or `unknown`.
   - Show each exact capability, event, `.hive/hooks/<capability>` path, executable command, and content digest.
   - Require approval per capability. Rejecting all hooks is valid and produces no hook artifact.
   - Explain that hooks perform only bounded data-integrity diagnostics and never prompt classification or rewriting, Skill activation, memory ingestion, subagent orchestration, or continuation.
   - A `Stop` event always returns a neutral allow result.
11. **Write preview**
    - Show files, marker edits, ignored SQLite paths, and any conflicts.

## Fixed product decisions

Do not ask about these:

- Local-only operation on an authenticated subscription host
- No provider API calls or provider API keys
- Markdown canonical knowledge/role/run data plus typed tracked YAML/TOML configuration and consent, with a rebuildable SQLite index
- Git tracking for non-confidential canonical files
- SQLite, WAL/SHM, generated backup, and runtime cache exclusion
- Seven-day maximum backup retention
- No Hive implementation of plan, Ralph, team, or provider session orchestration
- Orchestration owner selection: Codex prefers compatible OMX, Claude prefers compatible OMC, and every other supported case uses host-native capability
- Owner detection evidence sources: active-host capability metadata and side-effect-free public `--version` only

## Safety Invariants

- Never call a model-provider API or request an API key.
- Never write a consumer harness into the Hive source workspace.
- Never activate an optional Skill without explicit approval.
- Never activate a Skill whose consent payload or digest fails validation.
- Never claim persistent completion, subagents, or usage enforcement when the selected host does not expose the required capability.
- Never silently switch from OMX/OMC to another orchestration layer.
- Never offer or install fallback hooks unless the capability result is conclusively `absent`.
- Never treat `incompatible` or `unknown` as absence.
- Never allow a fallback `Stop` hook to continue or block a session.
- Never use Copier directly against the live consumer tree; Copier is an authoring and CI surface.
