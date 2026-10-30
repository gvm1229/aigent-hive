---
name: project-setup
description: (project-setup) Configure or reconfigure a local Hive harness only for an explicitly identified project, repository, folder, or path. Also handles an identified project's opt-in directive recovery after compaction. Route global user-scope setup, preference changes, and bare Hive setup to user-setup.
---

# Project setup through the CLI

Use only the explicitly identified consumer project and selected user root. Bare or global Hive
setup belongs to user-setup. Never put a consumer harness in a `hive-source.json` workspace.

## Start, run, verify

1. Inspect state once with `hive setup --target <project-root> --user-root <user-root> --show
   --summary --output json`. Use the same explicit roots throughout. Do not first read every
   project file, load the full schema, infer preferences, or build a YAML document. For an
   inspection-only request, report this state without starting setup.
2. If the host provides a terminal that the user can see and type into, run `hive setup --target
   <project-root> --user-root <user-root>`. The CLI collects missing global choices first when
   needed, then handles project mode, kind, language, roles, capabilities, preview and confirmation.
   A hidden tool PTY is not a user terminal. If none exists, give this one command and the reason;
   do not start an input wait or switch to repeated AI questions.
3. Resume incomplete work with the CLI's exact `next_action`. Do not restart answered questions.
   For supplied choices, copy `answer_patch_template` from the status result, keep `kind` and
   `schema_version` unchanged, fill only `answers`, and save the complete document as `<patch-file>`.
   A fresh project needs supplied `setup_mode`, `project_name`, `project_kind` and `primary_host`;
   the CLI supplies other defaults. Never invent choices or patch metadata. Existing complete
   answer files remain supported. For an authorized change, use the complete command:
   `hive setup --target <project-root> --user-root <user-root> --answers <patch-file> --capabilities <trusted-capabilities-file> --input-channel agent --apply --summary --output json`.
   Keep the explicit `--apply` mode. Use only trusted supplied or saved capability material;
   never fabricate a capability file to make apply succeed. Read
   [structured choices](references/structured-choices.md) only for unavailable capability material,
   custom overrides or separate consent. Separate consent requires the full `--dry-run` preview
   without `--summary` before apply; a summary never replaces approval scope.
4. The apply command validates project files and the registry before reporting completion.
   Check its status, exit code, `completion_verified`, capability limits and pending action,
   then verify stored state once with `--show --summary --output json`. These are the normal
   completion checks. Standalone file validation uses `--validate` with the same `--answers`,
   `--capabilities` and `--summary --output json`; use it for an explicit validation request or
   the CLI's failure-recovery action. Load generated files only for requested details or a failed
   verification. Keep completion within 8KiB and status within 4KiB. Optional pending questions
   do not mean failed installation, but pending, unsupported or failed operations stay incomplete.
   An interrupted AI reply calls for state inspection, never repeated apply.

The CLI owns the canonical question catalog. For an explicit question explanation, request only
`hive setup --target <project-root> --user-root <user-root> --describe --question <question-id>
--output json`. This is a localized, bounded question and source view, not approval to change it.
A full section/schema is an explicit diagnostic, not a normal setup prerequisite.

## Preserve authority

- Preserve user/third-party bytes, role bodies, assignments, handoffs and reviewed local overrides.
  The CLI's authenticated project-upgrade path handles older projections; do not replace files by hand.
- Saved selections do not prove actual host execution. Keep supported, best-effort, unsupported and
  unverified distinct. Never install hooks or optional Skills without their exact current consent.
- For an explicitly requested advanced choice, read [advanced constraints](references/advanced-setup.md).
  For instruction recovery after compaction, read only [directive recovery](references/directive-recovery.md).
  That separate request never restarts unrelated project setup.
- No provider APIs/credentials, host-global edits, foreign runtime inspection, or OMX/OMC invocation.
  Preserve existing owner pins. Never substitute another runtime or silently re-enable protection.
- No model/fast-mode changes, ordinary-setup Judge/subagents, repository-wide searches, Git commits
  or pushes under setup alone. A direct CLI setup makes zero provider-model calls.
- Return to any remaining authorized outer task after verified setup. A finished Skill is not a
  claim that the outer task is complete.

For optional model-choice advice in Codex, suggest available Luna low for quick project setup and
prefer normal speed. Evidence covers quick-setup accuracy, not custom-role reliability, measured
speed or cost savings. Do not recommend a model for unqualified routes. On other hosts, discuss
only their available light models; do not present Codex model names. Keep model and speed changes
under the user's control. The direct CLI needs no model.
