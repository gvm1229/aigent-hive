---
name: project-setup
description: (project-setup) Configure or reconfigure a local Hive harness only for an explicitly identified project, repository, folder, or path. Also handles an identified project's opt-in directive recovery after compaction. Route global user-scope setup, preference changes, and bare Hive setup to user-setup.
---

# Agent-driven project setup

Use only the explicitly identified consumer project and selected user root. Bare or global Hive
setup belongs to user-setup. Never put a consumer harness in a `hive-source.json` workspace.
Invoking this Skill means the user wants you, the AI agent, to run setup. You drive it in chat.
The interactive terminal `hive setup --target <project-root> --user-root <user-root>` is optional,
only when the user asks for it. A hidden tool PTY is not a user terminal; never wait on one.

## Inspect, decide, confirm, apply

1. Inspect both scopes once with the same explicit roots: `hive setup --scope user --user-root
   <user-root> --show --summary --output json` and `hive setup --target <project-root> --user-root
   <user-root> --show --summary --output json`. Without a user-named root, use the user's home
   folder and reuse the root echoed in `next_action`. For an inspection-only request, report and
   stop. If `agent_setup.user_setup_required` is true, finish global choices through user-setup
   first. An already configured project has no `agent_setup`: report its state, ask what to
   change, and patch only those keys. An unfinished terminal draft is superseded by your apply.
2. Empty folder (`agent_setup.project_state` is `empty`): the user is starting a new project.
   Interview in chat, one question per message: what the project is about, its goal, how they
   want it developed (stack, process, preferences), then anything still unclear. Never decide
   these silently. A plain request such as "set up hive for me" asks you to run setup; it never
   skips this interview, and no delegation phrase replaces it.
3. Existing project: scan before deciding, bounded to a structure listing, file-type counts and
   key manifests, README, build and config files. No repository-wide content reading, secrets or
   model calls. The CLI's `agent_setup.detected` facts are input, not the answer. Judge the kind
   from the evidence: a mountain of Markdown suggests an archive or library; TypeScript with React
   a web app; C++ with `.uasset` an Unreal game or an Unreal simulation, so look further. When
   still ambiguous, ask the user to confirm with your evidence.
4. Propose in one message: `setup_mode` (expedited unless the user wants custom), `project_name`
   (folder name unless the user named it), `project_kind` (a CLI `project-kind` choice decided
   from the interview or scan, not from `agent_setup.detected_defaults` alone), `primary_host`
   (the current host when it is in `selected_hosts`; otherwise ask, or add it through
   user-setup), and the interview or scan summary. Get confirmation or edits before apply.
   Only for an existing project with unambiguous scan evidence, an explicit "use your judgment"
   lets you apply the proposal and then report each value and how to change it.
5. Copy `answer_patch_template`, keep `kind` and `schema_version`, fill only `answers`, save it as
   `<patch-file>` in the system temporary folder (outside the project; delete it afterwards),
   preview it with `--dry-run` in place of `--apply` (writes nothing), then run the status `next_action`:
   `hive setup --target <project-root> --user-root <user-root> --answers <patch-file> --input-channel agent --apply --summary --output json`.
   No capability file is needed: `--capabilities` is optional for `--dry-run`, `--apply` and
   `--validate`, and the CLI uses the same saved or detected host capabilities as terminal setup.
   No command generates one. Pass `--capabilities` only with trusted supplied material; never fabricate one.
   A pending result (`hive.project-setup-input-pending`) lists `--show --summary` and the agent
   `--dry-run` in `next_action`; continue there, not in a terminal.
6. Check status, exit code, `completion_verified`, capability limits and pending action, then
   verify once with `--show --summary --output json`. Keep completion within 8KiB and status within
   4KiB. Pending, unsupported or failed operations stay incomplete. Standalone `--validate` takes
   the same `--answers` and is for an explicit request or the CLI's recovery action. An interrupted
   AI reply calls for state inspection, never repeated apply. Resume incomplete work with the
   CLI's exact `next_action`; do not restart answered questions.

After a verified apply, interview or scan facts that the setup fields cannot hold (purpose, goal,
development preferences) go into the user-editable `AGENTS.md` area outside Hive markers: show the
exact draft and obtain confirmation first, and preserve all existing user text. With Wiki enabled you may
also record them as project knowledge. Never invoke a provider API or a separate model process.

Read [structured choices](references/structured-choices.md) for patch details, custom overrides
or separate consent. Separate consent requires the full `--dry-run` preview without `--summary`
before apply; a summary never replaces approval scope. For an explicit question explanation,
request only `hive setup --target <project-root> --user-root <user-root> --describe --question
<question-id> --output json`. A full section/schema is a diagnostic, not a setup prerequisite.

## Preserve authority

- Never invent roles, hooks, optional Skills, knowledge scopes or thresholds. They stay empty
  unless the user explicitly chooses them. A rejection or cancellation preserves saved choices.
- Preserve user/third-party bytes, role bodies, assignments, handoffs and reviewed local overrides.
  The CLI's authenticated project-upgrade path handles older projections; do not replace files by hand.
- Saved selections do not prove actual host execution. Keep supported, best-effort, unsupported and
  unverified distinct. Never install hooks or optional Skills without their exact current consent.
- For an explicitly requested advanced choice, read [advanced constraints](references/advanced-setup.md).
  For instruction recovery after compaction, read only [directive recovery](references/directive-recovery.md).
  That separate request never restarts unrelated project setup.
- No provider APIs/credentials, host-global edits, foreign runtime inspection, or OMX/OMC invocation.
  Preserve existing owner pins. Never substitute another runtime or silently re-enable protection.
- No model/fast-mode changes, ordinary-setup Judge/subagents, Git commits or pushes under setup
  alone. A direct CLI setup makes zero provider-model calls.
- Return to any remaining authorized outer task after verified setup. A finished Skill is not a
  claim that the outer task is complete.

For optional model-choice advice in Codex, suggest available Luna low for quick project setup and
prefer normal speed. Evidence covers quick-setup accuracy, not custom-role reliability, measured
speed or cost savings. Do not recommend a model for unqualified routes. On other hosts, discuss
only their available light models; do not present Codex model names. Keep model and speed changes
under the user's control. The direct CLI needs no model.
