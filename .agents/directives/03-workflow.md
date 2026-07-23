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

- One commit contains one clear concern.
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

After every commit:

```bash
git log -1 --format=%B
```

Verify that the message has the intended scope and contains no co-author trailer.

## Push and Rewrite Safety

- Verify `git remote -v` and the target ref before pushing.
- Never force-push unless the user explicitly requests a history rewrite.
- When explicitly authorized, use `--force-with-lease`, never plain `--force`.
- Never delete `main` or `develop`.
- Do not push secrets, runtime state, caches, SQLite files, generated release output, or active-session manifests.

## Completion Boundary

Staging is not completion when the requested workflow includes commit or push. Finish the requested safe Git chain, then report all completed steps and any failure together.

Human-readable companion documents:

- [`../../docs/guides/commit-rules.md`](../../docs/guides/commit-rules.md)
- [`../../docs/guides/branching-rules.md`](../../docs/guides/branching-rules.md)
