---
name: user-setup
description: (user-setup) Configure global Aigent Hive preferences or answer new setup questions; use project-setup for a named project harness.
---

# Global setup through the CLI

The CLI owns questions, defaults, language, preview, application and recovery. This Skill starts
that flow and explains its result. Global setup never authorizes project inspection or setup.

1. Use the CLI-selected default user root or the already identified explicit user root. Keep it
   consistent; do not guess another root from the current repository. Inspect once with
   `hive setup --scope user --user-root <user-root> --show --summary --output json`.
   For inspection-only requests, report this state without starting setup. For executable discovery
   problems only, read [workflow](references/workflow.md).
2. Use a terminal that the user can see and type into for `hive setup --scope user --user-root
   <user-root>`. The CLI asks for an unset language first and reuses saved choices. A hidden tool
   PTY is not a user terminal. Without a user terminal, give this one command and the reason;
   do not wait for hidden input or replace the CLI with repeated AI questions.
3. For a named setting or already supplied choices, read only
   [reconfiguration](references/reconfiguration.md). Pass only the user's choices through the
   shared file/patch input with `--input-channel agent`. Never generate the full YAML after each
   answer, infer consent, or mark an Agent value as a direct human answer.
4. For incomplete work, use the CLI's exact `next_action` or the same-root `--resume` flow.
   Preserve its operation ID. An interrupted AI response is a reason to inspect status, not to
   apply again or restart the questionnaire. Monitor consent review uses the separate `--resume
   --review` flow; never silently revive cancelled monitoring.
5. Check the final status, exit code, saved choices, pending action and actual monitor evidence.
   Use `--summary --output json` for file apply/validate results: status is at most 4KiB and
   completion at most 8KiB. An enabled saved flag is not proof of a running monitor. Pending or
   failed work stays incomplete. Read [recovery](references/recovery.md) only for that failure.

The normal AI role is start and result explanation. Do not collect another questionnaire, translate
CLI questions, repeatedly fetch a schema, or launch a Judge/subagent for ordinary setup. Explicit
question explanations use [the question view](references/questions.md). Use the selected interface
language for explanations; [language](references/language.md) is only a writing reference.

New setups default blocking and interruption off; existing choices are preserved or explicitly
reviewed by the CLI. A bare continue/resume or urgency never authorizes an opt-out. Optional
downloads, external delivery, hooks and monitor interruption retain separate exact consent.
Do not use a summary in place of a complete approval preview.

Never call provider APIs, obtain credentials, read foreign runtime/session state or plugin caches,
edit host-global configuration, install CodexBar without consent, or change host model/fast settings.
Direct CLI setup makes zero provider-model calls. Optional AI explanations can use a light model
at normal speed, but specific host/model recommendations require actual qualification evidence.

When both scopes were requested, finish user setup and continue the named project through
project-setup. Otherwise do not inspect or create a project harness, compaction hook, Git commit
or push. Return control to the remaining authorized outer task after verified setup.
