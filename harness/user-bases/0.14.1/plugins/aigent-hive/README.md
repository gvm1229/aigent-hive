# Aigent Hive host plugin

Release-generated package for Codex, Claude Code, and Antigravity.

- Canonical Skill source: `harness/skills/`
- Generated Skill projection: `skills/`
- Codex manifest: `.codex-plugin/plugin.json`
- Claude Code manifest: `.claude-plugin/plugin.json`
- Antigravity manifest: `plugin.json`
- Claude status-line capture: opt-in `bin/hive-claude-usage-capture`

Existing Claude status line preservation:

```sh
sh -c 'payload=$(cat); printf "%s" "$payload" | hive-claude-usage-capture; printf "%s" "$payload" | /absolute/path/to/existing-status-command'
```

The composition duplicates the host-owned stdin only in process memory, keeps the
existing command's stdout as the status line, and never mutates
`~/.claude/settings.json`. Replace the absolute path before opting in through
Claude's `/statusline` command.

`scripts/sync-user-plugin.py` refreshes the generated Skill projection.
