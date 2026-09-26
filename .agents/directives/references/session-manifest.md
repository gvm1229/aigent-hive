# Session manifest template

```markdown
# Active Session: <session-id>

- Agent: <host or agent>
- Branch: <branch>
- Status: active | awaiting-user-authority | awaiting-external-evidence | blocked | complete
- Task: <summary>
- Started: <ISO-8601>
- Last updated: <ISO-8601>

## Remaining Agent-Owned Actions
- <action or none>

## Closure Evidence
- <evidence, owner, or none>

## Intended Edit Paths
- <project-relative path>

## Currently Edited Paths
- <project-relative path or none>

## Temporary Worktrees
- <absolute path | ref | purpose | removal boundary | status>

## Notes / Blockers
- <note or none>

## Commit Concerns
- <id>: <intent> | paths: <exact paths> | status: <state> | verification: <check>
```
