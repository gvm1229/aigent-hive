# Git Branches

## Branch Strategy

Maintain two default long-lived branches:

- `main` — stable, publicly releasable baseline
- `develop` — ongoing integration and ordinary development; ordinary fast-forward direct pushes
  are allowed

Use `release/staging` only with explicit user approval and a release-plan need. Require PRs,
status checks, deletion protection, and non-fast-forward protection.

Bootstrap rules:

1. Create the reviewed initial project commit on `main`.
2. Create `develop` from that commit.
3. Push both branches.

After bootstrap:

- Perform ordinary work on `develop`.
- Push ordinary verified commits directly to `develop`; do not require a pull request or required
  status checks for this branch.
- Do not commit ordinary work directly to `main`.
- Every branch merges to `develop` first; every `main` pull request has `develop` as its head.
  No feature, docs, test, or release branch merges directly to `main`.
- Stable authority approves only the final `develop → main` integration.
- Do not create named, purpose, feature, or snapshot branches under the default policy.
- Create another branch only when the user explicitly authorizes that exception for a specific task.
- An authorized non-default branch name must start with its work class: `feature/`, `fix/`,
  `release/`, `docs/`, `test/`, `refactor/`, `build/`, or `chore/`. Do not prefix a branch with
  an agent, model, assistant, or person name. Use the narrowest truthful work class.
- Use `scripts/branch-policy.py` for authorized branch creation and rename; see the guide below.

[Branch operation guide](../../../docs/guides/branching-rules.md).
