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
   - Follow [references/questions.md](references/questions.md).
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
