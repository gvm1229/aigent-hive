# 03. Git Workflow Directive

This directive governs Git branches, commits, staging, pushes, and repository history.

## Mandatory Preflight

Before any Git operation that stages, unstages, commits, switches branches, merges, rebases, tags, pushes, or rewrites history:

1. Reread this directive.
2. Run `git status --short --branch`.
3. Classify the operation as bootstrap, ordinary development, integration, release, or remote publication.
4. Inspect the exact paths and refs that will change.

## Branch Strategy

Maintain two default long-lived branches:

- `main` — stable, publicly releasable baseline
- `develop` — ongoing integration and ordinary development; ordinary fast-forward direct pushes
  are allowed

Create `staging` only when an explicit release plan needs a separate pre-production branch and the
user authorizes it. A created `staging` branch must use a strict ruleset with pull requests,
required status checks, deletion protection, and non-fast-forward protection.

Bootstrap rules:

1. Create the reviewed initial project commit on `main`.
2. Create `develop` from that commit.
3. Push both branches.

After bootstrap:

- Perform ordinary work on `develop`.
- Push ordinary verified commits directly to `develop`; do not require a pull request or required
  status checks for this branch.
- Do not commit ordinary work directly to `main`.
- Integrate `develop` into `main` through a pull request.
- Do not create named, purpose, feature, or snapshot branches under the default policy.
- Create another branch only when the user explicitly authorizes that exception for a specific task.
- An authorized non-default branch name must start with its work class: `feature/`, `fix/`,
  `release/`, `docs/`, `test/`, `refactor/`, `build/`, or `chore/`. Do not prefix a branch with
  an agent, model, assistant, or person name. Use the narrowest truthful work class.

## Temporary Worktree and Clone Lifecycle

- Use one primary worktree for ordinary work. Do not create a second worktree for convenience,
  visual separation, a separate concern, or a routine branch change.
- Create additional worktrees only when a workload is too large to complete safely in sequence and
  needs genuinely parallel independent changes. Record why one primary worktree cannot safely
  complete the workload before creation.
- A temporary worktree or clean-context clone is an isolation boundary for an authorized branch,
  protected-target PR, or exact-ref qualification. It is not a default organization mechanism or
  a substitute for ordinary commit splitting.
- Before creating one, record its exact absolute path, branch or detached ref, purpose, owner,
  and removal boundary in the active-session manifest. Do not create a worktree merely because
  concurrent editing might be convenient.
- A pushed PR branch is not a reason to retain its local worktree. After its concern has been
  committed, pushed, and verified, remove the clean local worktree in the same task with
  `git worktree remove <exact-path>`, then run `git worktree prune` and inspect
  `git worktree list --porcelain`.
- Do not use `--force`, remove the primary worktree, or remove a path not recorded as owned by
  the active session. A dirty temporary worktree or clone requires an owned commit and requested
  push, or an explicit retained-path report; never discard or silently leave its changes.
- Apply the same completion rule to disposable clean-context clones. Move a clone with retained
  local bytes to a recoverable location only after verifying its required commits or blobs are
  reachable from the intended remote ref.
- Before the final response, resolve every temporary path owned by the session as `removed` or
  `retained` with an exact reason. A failed cleanup remains an incomplete task boundary and must
  name the path and recovery action.
- Remove a completed temporary worktree immediately after its commits are reachable from the
  intended remote or retained primary ref and its required verification passes. Before removal,
  confirm that no uncommitted file, unreachable commit, or unpushed required commit remains.

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

## Test Artifact Lifecycle

- Before a local or CI test that produces source `tests/work/` or `target/debug/` output, use
  `python scripts/test-artifacts.py run --purpose <Korean-summary> --path <owned-path> --command <test-command>`.
- Keep the resulting `tests/results/runs/*.md` record. A passing test is not a cleanup authority
  until its result record is reviewed and committed.
- Use `python scripts/test-artifacts.py check` at a task closure. Resolve every eligible or expired
  item through an explicit review, then use `cleanup --apply` only for the exact reviewed paths.
- A path with a live process, a concrete 72-hour reuse reservation, a failed reproduction, or
  incomplete evidence remains retained. Never use a glob, parent-directory deletion, or age alone.

## Risk-Tier CI and Candidate Economy

- Match CI to the tracked diff. Markdown-only work uses documentation integration and starts no
  Rust, cross-platform, native package, runtime qualification, or candidate. Product work runs its
  affected lane, Linux conformance, and the smallest relevant macOS/Windows smoke; full platforms
  remain nightly or candidate evidence.
- CI for a superseded PR or branch commit must cancel when a newer commit for that same PR or ref
  starts. Never apply cancellation to release publication, accepted public tests, or protected
  stable candidate workflows.
- Cache only pinned dependencies per OS; never replace a test, lockfile contract, or artifact.
- At a completed user-authorized product milestone, automatically create, publish, and accept one
  next numbered test without another approval; intermediate commits never publish. Stable remains
  version-specific and explicit.
- Before a test candidate, `check-test-release-gate.py` must prove new shipped product bytes against
  `docs/public-test-product.json` and one or more checked non-release implementation plan IDs.
  Missing, stale, identical, source-only, or unplanned evidence refuses the candidate; fix the
  scope or evidence instead of asking the user to approve a release bypass.
- Plans, state, facts, docs, source-only Skills/directives, tests, CI, notices, and identical product
  trees never create or reset a test. Batch product work until milestone verification finishes.
- One always-run protected merge gate verifies risk-matched jobs; never require a conditionally
  skipped product job directly.

## Documentation-Only Integration

- Classify a change as documentation-only when every changed tracked file is Markdown. The only
  permitted non-Markdown companion is a static-contract test whose changed lines merely assert
  those Markdown paths or exact guidance tokens and execute no product behavior. The diff must
  contain no product source, workflow, package manifest, lockfile, schema, fixture, generated
  source, release artifact, or binary asset.
- Run every relevant local documentation gate before integration: Source Wiki index and lint for
  source facts, human-documentation style for human prose, Markdown link validation, and the
  nearest packaging or static-contract check when a shipped README or agent directive changes.
- After those relevant gates pass, unrelated full Rust, cross-platform, integration, security,
  and release CI may continue asynchronously and does not block merge. Record that it was not
  awaited; never report an unfinished check as passed.
- A completed failure in a relevant documentation, packaging, directive, secret, link, or allowed
  static-contract check remains blocking. Any other non-Markdown tracked change removes this
  exception and restores the ordinary verification tier.
- A documentation-only follow-up after publication does not create a new test version, stable
  candidate, tag, or package publication. It updates repository documentation only.

## Release Qualification Ordering

- Treat a successful `Release candidate` workflow as a private artifact-generation result only.
  It is not a published public test. A public test exists only after the separate publication
  workflow succeeds with that exact successful candidate run ID, and independent registry and
  GitHub Release checks confirm the requested version.
- Keep the candidate run ID, publication run ID, exact source SHA, package version, registry tag,
  and GitHub Release tag together in the release evidence. Never infer any of them from a workflow
  name, requested dispatch input, or a successful build job.
- A release workflow that runs a script which reads a historical Git tag or commit must use a full
  checkout history (`fetch-depth: 0`). Test this requirement at the workflow level. A shallow
  checkout failure must stop before artifact upload or registry publication; repair the workflow
  and use the next permitted numbered public test when product, package, installer, metadata, or
  acceptance bytes changed.
- Before reporting a numbered public test as complete, independently query the exact npm package
  version and channel tag plus the exact GitHub prerelease tag. Verify that `latest` remains
  unchanged for a test-channel publication. Report a failed candidate as unpublished, even when a
  dispatch used a numbered test version.
- Never publish or install a stable version as exploratory, regression, acceptance, performance,
  or final release testing. Stable publication is a terminal distribution action, not a test lane.
- Before creating a stable candidate, reconcile every active plan item. Complete every item in the
  release scope except a future-version candidate that the active plan explicitly defers by ID.
- Publish a uniquely numbered public test version from the qualified `develop` commit before the
  stable candidate. A local dev build, candidate artifact, CI result, or prior stable installation
  does not replace the numbered public test.
- Install the exact public test artifact on every required acceptance host and run the active
  plan's clean-install, upgrade, rollback, recovery, fresh-session, data-preservation, and
  performance checks. Bind the evidence to the test version, source commit, artifact digest,
  operating system, and actual execution result.
- Any product, packaging, installer, metadata, or acceptance fix invalidates earlier test evidence.
  Publish the next numbered test version and repeat affected acceptance checks; never repair the
  candidate by silently reusing a version or by testing through the stable channel.
- Create the protected `main` stable candidate only after the latest numbered test is accepted and
  the active plan reports zero incomplete in-scope items. Stable publication cannot create missing
  qualification evidence or be described as testing.

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
- Never delete `main`, `develop`, or an active release `staging` branch.
- Do not push secrets, runtime state, caches, SQLite files, generated release output, or active-session manifests.

## Completion Boundary

Staging is not completion when the requested workflow includes commit or push. Finish the requested safe Git chain, then report all completed steps and any failure together.

Human-readable companion documents:

- [`../../docs/guides/commit-rules.md`](../../docs/guides/commit-rules.md)
- [`../../docs/guides/branching-rules.md`](../../docs/guides/branching-rules.md)
