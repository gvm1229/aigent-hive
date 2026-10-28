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
   For choices already supplied by the user, use the existing file input with
   `--input-channel agent`, or the small answer-patch format described in
   [structured choices](references/structured-choices.md). Copy `answer_patch_template` from the
   status result, keep `kind` and `schema_version` unchanged, and fill only `answers`.
   Use only trusted supplied or saved
   capability material; never fabricate a capability file to make apply succeed.
4. Read an apply/validate result with `--summary --output json`. Check its status, exit code,
   actual capability limits and pending action. The normal completion response is at most 8KiB;
   state is at most 4KiB. An optional pending question is not a failed installation. A pending,
   unsupported or failed operation is not complete. Verify state with
   `--show --summary --output json`, never with the full `--show` response during normal setup.
   Never repeat apply merely because an AI reply was interrupted.

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
