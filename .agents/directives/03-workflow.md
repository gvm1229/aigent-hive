# 03. Git and verification

Before staging, unstaging, committing, switching, merging, rebasing, tagging, pushing or rewriting:
read the applicable row once (again if changed), run `git status --short --branch`, classify the
operation as bootstrap/development/integration/release/publication, and inspect exact paths/refs.
Ordinary work stays on develop; branch exceptions need explicit authority.

| Action | Required reference |
| --- | --- |
| Stage/commit | [Commits](references/git-commits.md) |
| Branch creation/rename/bootstrap/integration | [Branches](references/git-branches.md) |
| Extra worktree/clone | [Worktrees](references/git-worktrees.md) |
| Product tests/qualification | [Verification](references/verification.md) |
| Source-only documents/checkers | [Source verification](references/documentation-verification.md) |
| CI changes/product milestone | [CI/candidates](references/ci-and-candidates.md) |
| Candidate/public test/stable/publication | [Release](references/release-qualification.md) |

Do not eagerly load every row. After each commit inspect `git log -1 --format=%B` for scope and
no AI co-author trailer. Verify remote and target ref before pushing. No history rewrite without
explicit authority; use --force-with-lease, never plain --force, when authorized. Do not rewrite
old commits just to apply today's splitting policy. Never delete main, develop or active
release/staging; never push secrets, runtime/cache/SQLite, release output or session manifests.
Finish the requested safe Git chain; staging alone is not completion. Report failures with completed steps.
