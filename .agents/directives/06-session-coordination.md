# 06. Session Coordination Directive

This directive governs concurrent agent edits.

## Manifest

Before editing tracked files, create:

```text
.agents/work/active-sessions/<session-id>.md
```

The manifest is local scratch state and must never be committed.

Required fields:

```markdown
# Active Session: <session-id>

- Agent: <agent or host>
- Branch: <branch>
- Status: planning | editing | blocked | done
- Task: <summary>
- Started: <ISO-8601>
- Last updated: <ISO-8601>

## Intended Edit Paths

- path/from/repository/root

## Currently Edited Paths

- path/from/repository/root

## Notes / Blockers

- <optional note>

## Commit Concerns

- <concern-id>: <intent> | paths: <exact paths or bounded families> | status: pending | editing | verified | committed | verification: <nearest check>
```

## Conflict Check

1. Read all active manifests.
2. Ignore only manifests explicitly marked `done`.
3. Compare exact paths and parent/child directory scopes.
4. Stop before editing when another active session overlaps.
5. Update the manifest when scope changes.
6. Mark it `done` or delete it after completion.

## Commit Coordination

1. Assign every edited tracked path to exactly one commit concern before mutation.
2. Keep each delegated editing scope inside one concern. Record the concern ID, owned paths, and
   nearest verification in the primary session manifest.
3. Integrate, verify, and commit a completed concern before opening or delegating the next
   independent concern.
4. When agents must touch a shared foundational file, serialize that file under one concern and
   commit the foundation before dependent concerns.
5. Do not use concurrent editing, a shared milestone, or a final full-suite gate as a reason to
   accumulate independently revertible work in one uncommitted worktree.

This is advisory coordination. Serialize overlapping edits under the default two-branch policy. Use another branch or worktree only when the user explicitly authorizes that exception.
