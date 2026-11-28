---
name: ai-slop-cleaner
description: (ai-slop-cleaner) Remove AI slop from code, documents or a finished design the user selects. Shows a short fix plan and waits for confirmation before any edit. Formerly code-polish. Use only on explicit cleanup, de-slop or simplify requests.
---

# AI Slop Cleaner

Take AI tells out of work that already exists: code, documents or a built design. Keep what it
means and does. Formerly `code-polish`; when a user names `code-polish`,
say the new name `ai-slop-cleaner` and continue. Setting a new design direction belongs to
`design-curate`, and Korean sentence rewriting belongs to `humanize-kor`.

## Read order and precedence

1. Recorded user preferences first: recall Hive knowledge for `design/*` and writing topics that
   apply, if the Wiki is enabled. They win over every file below and survive upstream updates.
2. [references/hive-overrides.md](references/hive-overrides.md): where Hive differs from the
   bundled sources.
3. The one target checklist: [code](references/code.md), [writing](references/writing.md) or
   [design](references/design.md). Read only the one that matches the target.
4. Bundled sources under `references/upstream/` only when the checklist points there.

## Procedure

1. **Plan first; confirm before editing.** Inspect the scope and list each finding, how you will
   fix it and why, in a few short lines. Show the plan to the user and edit nothing until they
   confirm. The user removes anything that should stay. Under `autonomous-workflow`, treat this
   confirmation as an approval step: prepare the plan, keep working on other tasks, and put the
   plan in the questions asked at the end.
2. **Bound the scope.** Use the files or selection the user named; otherwise the changes on the
   current branch, including uncommitted ones. Report findings outside the scope; do not edit
   them.
3. **Lock what must not change.** Code: observable behavior, with a regression baseline run before
   editing. Documents: meaning, numbers, conditions and sources. Design: brand, accessibility and
   responsive states. No baseline or fallback check means no edit.
4. **Fix only confirmed items**, one smell class per pass, and run the target's check after each
   pass. Stop on a regression, a widened contract, a new dependency or an unclear effect.
5. **Report** changed files, removed items, checks run and remaining risks.

Review mode: when the user asks only for a review, stop after step 1 and return the plan as the
verdict.

## Shared smell classes

- Repetition: the same logic, sentence or block more than once.
- Padding: narration, filler, empty summaries, decoration without a job.
- Needless layers: wrappers, hedges, a card inside a card.
- Generic defaults: the stock choice made without a reason.
- Unsupported claims: invented numbers, names or results.

## Boundaries

- Prefer deletion and existing conventions over a new abstraction.
- Never edit generated, vendored, licensed, secret-bearing or out-of-scope files.
- Never hide a failing baseline or claim an improvement without fresh evidence.
