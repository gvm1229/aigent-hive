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
```

## Conflict Check

1. Read all active manifests.
2. Ignore only manifests explicitly marked `done`.
3. Compare exact paths and parent/child directory scopes.
4. Stop before editing when another active session overlaps.
5. Update the manifest when scope changes.
6. Mark it `done` or delete it after completion.

This is advisory coordination. Serialize overlapping edits under the default two-branch policy. Use another branch or worktree only when the user explicitly authorizes that exception.
