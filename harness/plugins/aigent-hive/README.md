# Aigent Hive host plugin

Release-generated package for Codex, Claude Code, and Antigravity.

- Canonical Skill source: `harness/skills/`
- Generated Skill projection: `skills/`
- Codex manifest: `.codex-plugin/plugin.json`
- Claude Code manifest: `.claude-plugin/plugin.json`
- Antigravity manifest: `plugin.json`
- Claude status-line capture: opt-in `bin/hive-claude-usage-capture`

Recommended Claude status-line connection (consented, reversible):

```sh
hive setup --scope user --section usage --claude-statusline preview
hive setup --scope user --section usage --claude-statusline apply --confirm <approval_digest>
hive setup --scope user --section usage --claude-statusline revert
```

The preview shows the exact `statusLine` change. Apply changes only the `statusLine`
member of `~/.claude/settings.json`, and only with the approval digest from the preview.
An existing status-line command keeps running: the connected command is
`hive usage statusline --host claude --pass-through | <existing command>`, so the
existing command receives the same input and its output stays the status line.
`revert` restores the previous value. A rejected capture never changes the status
line; `hive usage status` and the enforce warning show its reason.

Limit: status-line execution in the Claude desktop app Code tab is not verified.
If no measurement arrives there, use Claude Code in a terminal. Work continues with a
missing-measurement warning.

Manual composition without the setup command:

```sh
sh -c 'payload=$(cat); printf "%s" "$payload" | hive-claude-usage-capture; printf "%s" "$payload" | /absolute/path/to/existing-status-command'
```

`scripts/sync-user-plugin.py` refreshes the generated Skill projection.
