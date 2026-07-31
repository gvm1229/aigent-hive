---
name: hive-commit
description: Split source changes into focused, verified Git commits when the user asks to commit, ship, stage and commit, or separate changes by concern. Do not use for push-only requests or history rewrites unless explicitly requested.
---

# Hive Commit

Commit the requested source changes without mixing independently reviewable or revertible concerns.

## Workflow

1. Read the repository Git workflow directive and human commit guide.
2. Run `git status --short --branch`; inspect the full unstaged and staged diff plus recent
   human-authored commit style.
3. Build a concern map before staging. For every intended commit, list:
   - one intent;
   - exact paths or hunks;
   - nearest verification.
4. Split concerns by independent review and rollback. Treat Wiki or documentation state, product
   behavior, tests for another feature, version metadata, release date, and release activation as
   separate by default.
5. Stage one concern only. Use patch staging or edit sequencing when one file contains multiple
   concerns.
6. Inspect `git diff --cached`, then run `git diff --cached --check`,
   `git diff --cached --stat`, and the nearest affected verification.
7. Commit using the repository message style. Never add an AI, bot, agent, OMX, Codex, Claude, or
   Gemini co-author trailer.
8. Run `git log -1 --format=%B`, verify scope and trailer rules, then repeat for the next concern.
9. Push only when the user requested it. Follow repository pre-push verification and remote-ref
   checks first.

## Safety Boundaries

- Do not stage unrelated user changes.
- Do not treat staging as completion when the user requested a commit.
- Do not combine concerns merely because they came from one task or share a file.
- Do not rewrite, amend, rebase, or split existing history unless the user explicitly requests
  that history change.
- Never bypass repository hooks or verification with low-level Git plumbing.
