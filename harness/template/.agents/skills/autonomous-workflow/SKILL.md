---
name: autonomous-workflow
description: "(autonomous-workflow) Get the work done without stopping to ask about every small thing. Decide trivial choices yourself, keep going until everything doable is done, and ask only the questions you truly cannot answer, once, at the end. Use when the user asks to work autonomously, not to stop midway, to just get it done, or to save questions for the end."
---

# Autonomous Workflow

The user wants the work finished while they do something else. Every mid-task question pulls them
back. **Do not halt and ask midway.** Decide, keep going, and ask at the end.

## Rules

1. **Never stop midway to ask.** Work until every step you can finish alone is finished. A progress
   update is not a stopping point; do not end a turn just to report.
2. **Decide small things yourself.** If an answer follows from the request, the code, the project
   rules, recorded preferences or common convention, or is cheap to undo, choose it and move on.
   Naming, file placement, formatting, equivalent library choices, test layout and wording are
   yours to decide.
3. **Park real questions and keep working.** A real question is one only the user can answer: a
   material product choice, information you cannot find, or an action needing their authority.
   Add it to a list, take a safe provisional path if one exists, and continue all work that does
   not depend on it. Hold only the steps that truly depend on the answer.
4. **Ask once, at the end.** Finish with:
   - what was done and how it was verified;
   - choices you made yourself, one line each, so any can be reversed;
   - the parked questions, each self-contained, with your recommended answer first;
   - the steps waiting on those answers.

## Limits autonomy does not remove

- Approval-required actions stay approval-required: sending messages, publishing, purchases,
  moving money, account or security changes, permanent deletion, and anything the host or project
  marks as needing approval. Prepare them fully and list them at the end instead of doing them.
- Stop immediately when going on is unsafe: usage halt, user cancel or interrupt, a login only the
  user can do, or a destructive step without a backup.
- Never weaken a check, skip a test or quietly shrink scope to avoid a question.
- Report failures, skips and unverified results honestly in the final summary.
