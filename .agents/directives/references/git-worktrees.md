# Git Worktrees

## Temporary Worktree and Clone Lifecycle

- Maintainer sessions may create as many temporary worktrees as the job needs for parallel or isolated work; this replaces the convenience/sequential-work restrictions (maintainer, 2026-11-12).
- A temporary worktree or clean-context clone isolates an authorized branch, protected-target PR or exact-ref qualification; it does not replace ordinary commit splitting or grant branch authority.
- Before creation, record exact absolute path, branch/detached ref, purpose, owner and removal boundary in the active-session manifest.
- After work merges or ends, verify reachability and required pushes, then run `git worktree remove <exact-path>`, delete its merged branch locally and on origin, run `git worktree prune`, and inspect `git worktree list --porcelain`; leave no owned temporary worktree at closure (maintainer, 2026-11-12).
- A pushed PR branch does not justify retaining its local worktree. Unmerged retained commits need a safe reachable ref; never discard them to satisfy cleanup.
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
