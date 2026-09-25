# Git Commits

## Commit Rules

- Commit every completed source task independently before combining it with later completed work.
  Never bundle unrelated completed tasks in one commit, even when they share a request, session,
  milestone, or delivery window. Split a task further when it contains independently reviewable
  and revertible concerns.
- One commit contains one clear concern.
- Define a concern by an independently reviewable and revertible intent, not by file proximity,
  shared task origin, or the convenience of one staging operation.
- Split documentation or Wiki state, product behavior, version metadata, release activation, and
  unrelated test infrastructure by default. Combine them only when they are mechanically
  inseparable and explain that dependency in the commit body.
- A Wiki capture and a product version or release-date change are separate commits.
- Before staging, enumerate the intended commits and the exact paths or hunks owned by each.
- When one file contains multiple concerns, use patch staging or sequence the edits so each commit
  contains only its own hunks.
- Stage only the files required for that concern.
- Inspect recent human-authored commit style first. If none exists, use:

```text
<type>: <concise Korean description>
```

- Use Conventional Commit types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `delete`, `revert`.
- Use concise Korean titles without sentence-ending punctuation.
- Keep technical identifiers in English where clearer.
- For non-trivial commits, use concise file/path-scoped body bullets.
- Never add `Co-Authored-By` trailers for an AI, bot, agent, OMX, Codex, Claude, or Gemini.

Before every commit:

```bash
git status --short
git diff --cached --check
git diff --cached --stat
```

Run the nearest verification for the staged concern before committing. A later broader suite does
not make an internally mixed commit acceptable.

## Iterative Commit Checkpoints

- Treat local commits as ordinary implementation boundaries, not publication. A request to
  implement through multiple milestones authorizes the local commits required by this directive
  unless the user explicitly forbids commits. A prohibition on publishing, releasing, tagging, or
  pushing does not prohibit local commits.
- Before starting the next independently reviewable concern, verify and commit the completed
  concern. Do not postpone completed concerns until every checklist item, security review, or
  milestone in a larger plan is complete.
- An unfinished concern may remain uncommitted only when its nearest verification cannot run until
  mechanically inseparable work is complete. Record that dependency in the active-session manifest
  and do not begin an unrelated concern while it remains unresolved.
- At each checkpoint, count exact changed files with `git status --porcelain=v1 -uall` and refresh
  the concern map. When the worktree exceeds 50 changed files or contains more than one concern,
  stop new edits and commit every completed concern before continuing.
- A single generated or projected concern may exceed 50 files only when the concern map records
  its canonical source, every projection family, and the verification proving source-to-projection
  parity. File count never justifies combining independent concerns.
- A final full-suite result supplements checkpoint verification; it never replaces concern-local
  verification or local commits.
