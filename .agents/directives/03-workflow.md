# 03. Git Workflow Directive

This directive governs Git branches, commits, staging, pushes, and repository history.

## Mandatory Preflight

Before any Git operation that stages, unstages, commits, switches branches, merges, rebases, tags, pushes, or rewrites history:

1. Reread this directive.
2. Run `git status --short --branch`.
3. Classify the operation as bootstrap, ordinary development, integration, release, or remote publication.
4. Inspect the exact paths and refs that will change.

## Branch Strategy

Maintain exactly two long-lived branches:

- `main` — stable, publicly releasable baseline
- `develop` — ongoing integration and ordinary development

Bootstrap rules:

1. Create the reviewed initial project commit on `main`.
2. Create `develop` from that commit.
3. Push both branches.

After bootstrap:

- Perform ordinary work on `develop`.
- Do not commit ordinary work directly to `main`.
- Integrate `develop` into `main` through a pull request.
- Do not create named, purpose, feature, or snapshot branches under the default policy.
- Create another branch only when the user explicitly authorizes that exception for a specific task.

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

## Verification Tiers

Match verification cost to the current boundary:

1. **Work loop** — run the changed Rust crate tests and directly related Python tests only.
2. **Pre-commit** — run affected crates plus the nearest black-box, schema, static-contract,
   or regression tests for the changed behavior.
3. **Pre-push** — run the full Rust workspace and full Python conformance suite once for the
   logical milestone being pushed. Do not repeat an unchanged full-suite result for every
   commit in the same milestone.
4. **Release** — run clean-clone CI, every supported OS/architecture, hostile and security
   suites, installer/update recovery, signing, provenance, and publication qualification.

Keep existing hostile and security tests. Until the first public release, do not add a new
hostile edge-case implementation or test unless it directly protects installation, canonical
data, credentials, external-path confinement, update rollback/recovery, or a regression found
in the changed behavior. Record other hardening candidates for post-release review instead of
expanding the active implementation.

After every commit:

```bash
git log -1 --format=%B
```

Verify that the message has the intended scope and contains no co-author trailer.

## Push and Rewrite Safety

- Verify `git remote -v` and the target ref before pushing.
- Never force-push unless the user explicitly requests a history rewrite.
- Do not rewrite an existing commit solely to apply current commit-splitting policy unless the user
  explicitly requests that history change.
- When explicitly authorized, use `--force-with-lease`, never plain `--force`.
- Never delete `main` or `develop`.
- Do not push secrets, runtime state, caches, SQLite files, generated release output, or active-session manifests.

## Completion Boundary

Staging is not completion when the requested workflow includes commit or push. Finish the requested safe Git chain, then report all completed steps and any failure together.

Human-readable companion documents:

- [`../../docs/guides/commit-rules.md`](../../docs/guides/commit-rules.md)
- [`../../docs/guides/branching-rules.md`](../../docs/guides/branching-rules.md)
