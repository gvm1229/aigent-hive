---
name: user-setup
description: (user-setup) Configure global Hive preferences or answer new setup questions; use project-setup for a named project harness.
---

# Global Aigent Hive setup

Use the installed CLI to configure user-scope preferences. Preserve existing answers and project
boundaries. Version output locates an executable; release ownership checks authenticate mutations.

## Choose the requested operation

- First setup: read [workflow](references/workflow.md) and [initial questions](references/questions.md).
- A named setting or selected settings: read [reconfiguration](references/reconfiguration.md).
  Ask about the named settings directly; do not first display or restart the entire questionnaire.
- Interrupted setup: inspect saved progress, then read the applicable workflow section.
- New feature question: read workflow step 15; preserve previous answers and their explicit refusals.
- An installation problem: read [recovery](references/recovery.md). Do not infer ownership from a
  syntactically valid manifest or invoke an unverified removal as a repair.
- Localized questions: consult [language](references/language.md) only when composing those choices.
- Apply/validate: use workflow steps 9–14 with the same answers and exact user root.

Read only the relevant sections. Reference instructions about other branches are not prerequisites.
The signed CLI's describe result owns current fields, choices, and defaults.

## Authority and completion

Global setup never authorizes inspecting or configuring a project. When both scopes were explicitly
requested, finish global setup and continue the already authorized named project through project-setup.
Ask only for missing target information, new authority, or a material unresolved user choice.
An existing explicit choice remains valid during continuation and safe retries.

A validated, conflict-free built-in setup can apply under the existing setup request. Optional
downloads, external integrations, destructive operations, and changes of scope keep their own
explicit consent requirements. Never infer a yes/no answer from silence or cancellation.
Finish by validating the installed result. A completed Skill step is not completion of an unfinished
outer task; return control to the already authorized workflow.

## Safety Invariants

- Operate only at user scope. Never add or update a project harness.
- Use `hive install --scope user --host <host>` only for the authenticated Hive-file refresh
  prerequisite; use `hive setup --scope user` for all preference state changes.
- Never call a model-provider API or request, read, store, or forward provider credentials.
- Never read or mutate `.omx/`, `.omc/`, provider runtime state, or host-global configuration.
- Never activate an optional third-party Skill through this workflow.
- Never install CodexBar without exact current-action consent.
- Never commit or push unless the user explicitly requests that Git operation.
