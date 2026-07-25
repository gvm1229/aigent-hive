---
name: hive-run-resume
description: Read and validate a durable Hive run through the signed Hive CLI and prepare bounded fresh-session recovery data without changing project files. Use when resuming a specific `.hive/runs/RUN_ID/` after compaction, handoff, or a new session; do not use for simple questions, plan creation, automatic continuation, runtime spawning, or persistent execution loops.
---

# Hive Run Resume

Recover provider-neutral state only. This Skill never starts work or launches the resolved orchestration owner.

## Workflow

1. Run `hive run resume --help`. If unavailable, report the installed release as unsupported. Do not reconstruct resume behavior manually.
2. Obtain a fresh normalized capability-resolution JSON from the active host adapter. Do not inspect `.omx/`, `.omc/`, plugin caches, session state, or host-global configuration.
3. Run exactly one bounded read:

   ```text
   hive run resume --target <project-root> --run <run-id> --capabilities <fresh-capability-resolution.json> --output json
   ```

4. Require a schema-valid result bound to the exact PLAN, STATUS revision, active role documents, shared role handoff entries, evidence bytes, and immutable owner evidence.
5. Handle the result without hidden transitions:
   - For `executing` or `verifying` with `supported` or `best-effort` subagent support, return the prepare-only dispatch briefs to the already resolved host owner. Do not spawn or invoke it.
   - For `unsupported` or `unverified`, stop on exit code `4`; no dispatch brief is authorized.
   - For `blocked` or `usage-limited`, return recovery data and the resume condition without dispatch.
   - For `resume-ready`, return recovery data only. A later explicit host-owned action may transition the run.
   - For terminal states, report the durable result without continuing.
6. State the pinned owner, revision, next action, blocker or resume note, active roles, and verified evidence locators.

## Boundaries

- Keep the simple-question path isolated; do not load this Skill for self-contained answers.
- Never write STATUS.md, PLAN.md, role files, handoffs, evidence, or any other project file.
- Never create a plan, Ralph loop, team workflow, retry loop, subagent, model call, provider API request, runtime process, or automatic continuation.
- Never select, replace, install, configure, or invoke OMX/OMC.
- Never fall back to another owner when current capability evidence is missing, incompatible, unknown, or changed.
