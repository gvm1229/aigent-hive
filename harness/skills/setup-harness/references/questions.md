# Setup question sequence

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
6. **Usage guard**
   - Default stop threshold: 10% remaining.
   - Explain that the threshold is enforceable only when a fresh local subscription usage sensor exists.
   - Without a trustworthy sensor, automatic continuation must fail closed.
7. **Judge policy**
   - Normal risk: one independent judge when requested.
   - Elevated risk: two of three independent judges.
   - Critical risk: three of three plus human approval.
8. **Optional Skills**
   - Present each candidate separately with provenance and overlap.
   - Present requested filesystem, shell, network, subagent, and external-app capabilities.
   - Require a direct approval or rejection for each capability and candidate.
   - Sort capability IDs and record `sha256(RFC 8785 JCS(payload))` over consent version, exact displayed name/source/revision/content digest, requested/approved capabilities, and UTC-seconds approval time.
9. **Optional fallback hooks**
   - Ask only when the automatic external capability result is conclusively `absent`.
   - Do not ask when the result is `available`, `incompatible`, or `unknown`.
   - Show each exact capability, event, `.hive/hooks/<capability>` path, executable command, and content digest.
   - Require approval per capability. Rejecting all hooks is valid and produces no hook artifact.
   - Explain that hooks perform only bounded data-integrity diagnostics and never prompt classification or rewriting, Skill activation, memory ingestion, subagent orchestration, or continuation.
   - A `Stop` event always returns a neutral allow result.
10. **Write preview**
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
