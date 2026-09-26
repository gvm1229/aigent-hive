# Run Closure

## Final Response Closure Gate

Apply the status meanings from `01-behavior.md`:

1. Reconcile the request, owning checklist, worktree, evidence, and authorized remote actions.
   Record user-directed priority changes; a completed subtask does not complete its suspended parent.
2. Record remaining `agent-owned`, `awaiting-user-authority`, `awaiting-external-evidence`, and
   `blocked` items in the active-session manifest.
3. Continue execution when any `agent-owned` item remains.
4. For a non-agent-owned item, record exact action or evidence, owner, and why the active session
   cannot obtain it. Reflect a material handoff in `CURRENT.md`.
5. Claim completion only when the scoped criteria and evidence are complete.

Record target, ID, revision/digest, initialize/validate receipts, and source-criterion
mapping in the manifest. Before completion, call
`hive run closure --target <bound-target> --run <run-id> --output json`: inspect
`data.closure.ready_for_final` and remaining criteria, not exit zero or the command code.
Cancellation is not success; `global_block_permitted` is not authority. Recover the same run and
revalidate changed evidence. Stale or unrelated receipts cannot close it.

No consumer `.hive/` at source roots. Reviewed `tests/work/` runners must map to source criteria;
the source plan owns closure. Without a binding, use this plan/session procedure and
continue authorized work without claiming verified activation.
