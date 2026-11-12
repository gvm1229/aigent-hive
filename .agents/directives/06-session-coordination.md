# 06. Session Coordination Directive

Owns session manifests, edit-path reservations and concurrent-agent conflict checks. Git, commit
and worktree lifecycle: `03-workflow.md`.

## Manifest

Before tracked edits, create/resume `.agents/work/active-sessions/<session-id>.md` with
[manifest fields](references/session-manifest.md). Load the template at creation; reuse for updates.

Ignored runtime state; never canonical knowledge or committed.

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

Worktree authority, lifecycle, cleanup and commit sequencing: `03-workflow.md` and its Worktrees
reference; record temporary worktrees here.

## Closure record

- Keep status `active` while `01-behavior.md` classifies any scoped action as agent-owned.
- Before final response, refresh actions and closure evidence; an active manifest prohibits completion claims.
- `awaiting-user-authority`/`awaiting-external-evidence`: exact owner and expected proof.
- `blocked`: 01's run-wide repeated condition and recovery path.
- Before closure, resolve each temporary worktree as removed or retained with an exact reason.
