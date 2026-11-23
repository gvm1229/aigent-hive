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

## Automatic knowledge lookup (Claude Code)

The Claude Code manifest registers one `UserPromptSubmit` hook:
`hive knowledge prompt-context --stdin-json`. For each prompt it runs one local lookup
(top 5, 4 KiB) with the prompt text and the session folder, and adds the matches as
context. It writes nothing, never stores the prompt, and never blocks the prompt: a
disabled Global Wiki, a missing `hive` command, an error, or no match adds nothing.
The hook comes with the plugin; it does not change `~/.claude/settings.json`. To stop
it, disable the plugin. Codex and Antigravity do not have this hook.

## Message hook setup

`hive msg hooks apply` changes only the current project's hook file after the preview digest is
confirmed. For Codex, trust the project hooks when Codex shows its trust prompt. For Claude Code
automatic mode, a project permission rule for `Bash(hive msg:*)` in `.claude/settings.json` is your
choice. A `Bash(codex exec:*)` rule removes prompts for every Codex run, including unsafe flags;
prefer approving each run. Check the recorded hook state
with the `hive msg hooks status` command shown after apply; that status does not prove the host
loaded or ran the hook.

## Compatibility

Claude Code resolves `/plugin:skill` commands itself. Hive cannot redirect an old slash-command
name; use the current Skill name shown by the old-name guidance.

Limit: status-line execution in the Claude desktop app Code tab is not verified.
If no measurement arrives there, use Claude Code in a terminal. Work continues with a
missing-measurement warning.

Manual composition without the setup command:

```sh
sh -c 'payload=$(cat); printf "%s" "$payload" | hive-claude-usage-capture; printf "%s" "$payload" | /absolute/path/to/existing-status-command'
```

`scripts/sync-user-plugin.py` refreshes the generated Skill projection.
