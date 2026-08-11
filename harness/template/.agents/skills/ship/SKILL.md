---
name: ship
description: Prepare focused, verified commits using the current repository rules. Push only when the user explicitly asks.
---

# Ship changes

Use this Skill when the user asks to commit, ship, stage and commit, or split a change set.

1. Read the repository's `AGENTS.md` and any referenced Git or commit guide.
2. Inspect status, the complete diff, and recent commit style before staging.
3. Map each independently reviewable concern to exact files or hunks and its nearest verification.
4. Stage one concern at a time, inspect the staged diff, run the mapped verification, and create a focused commit using the repository's style.
5. Push only when the user explicitly authorizes it. Never rewrite history, bypass hooks, or stage unrelated changes.

## Boundaries

- Do not embed a repository's branch names, release policy, or version rules in this Skill.
- Do not create a commit merely because files are staged.
- Do not use provider credentials, invoke a provider API, or edit foreign files.
