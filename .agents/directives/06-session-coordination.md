# 06. Session Coordination Directive

Owns active-session manifests, edit-path reservations, and concurrent-agent conflict checks. Git,
commit, and worktree lifecycle rules belong to `03-workflow.md`.

## Manifest

Before editing tracked files, create or resume `.agents/work/active-sessions/<session-id>.md` with:

[the manifest fields](references/session-manifest.md). Load the template when creating a manifest;
reuse the existing record for updates.

The manifest is ignored runtime state, never canonical knowledge or a commit.

## Conflict check

1. Read every manifest not marked `done` or `complete`.
2. Compare exact paths and ancestor/descendant scopes.
3. Stop before an overlapping automated write from another live session.
   A saved `active` label or old date alone does not prove liveness. Inspect available host/session
   evidence first. If a session is verified ended, retain its work and close only its stale path
   reservation; this does not mark its product criteria complete. If liveness is ambiguous, pause
   overlapping writes only and continue independent work. Ask only when the evidence cannot be resolved.
4. Update this session's reservation before adding a path.
5. Assign each edited path to exactly one commit concern.
6. Serialize a shared foundational file; do not let agents edit it concurrently.

Use one primary worktree. Additional worktree authority, lifecycle, cleanup, and commit sequencing
come only from `03-workflow.md`; record any authorized temporary worktree here.

## Closure record

- Keep status `active` while `01-behavior.md` classifies any scoped action as agent-owned.
- Before a final response, refresh remaining actions and closure evidence. An active manifest
  prohibits a completion claim.
- `awaiting-user-authority` and `awaiting-external-evidence` require exact owner and expected proof.
- `blocked` requires the run-wide repeated condition and recovery path defined by `01-behavior.md`.
- Resolve every temporary-worktree entry as removed or retained with an exact reason before closure.
