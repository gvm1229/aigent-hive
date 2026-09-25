# Git Worktrees

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
