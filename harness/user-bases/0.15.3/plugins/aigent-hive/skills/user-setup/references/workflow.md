# CLI entry and continuation

Use the installed signed CLI. A version string locates a program; ownership checks authorize its
mutations. If `hive` is not available, use this Skill's `../scripts/resolve-hive.ps1` on Windows,
or `command -v hive` on POSIX. Keep the returned executable and the same selected user root.
Do not recursively search package/plugin folders or recreate the renderer manually.

Inspect state once: `hive setup --scope user --user-root <user-root> --show --summary --output json`.
The agent asks the user's choices in chat and applies them with `--answers <patch-file>
--input-channel agent --apply`. The terminal `hive setup --scope user --user-root <user-root>` is
optional, only on request; a tool-only PTY does not supply human input. The CLI validates choices,
previews, applies, activates apps and validates. It preserves prior choices, unknown migration origins and foreign bytes.

For an interrupted operation use its exact resume command. Do not replay setup after an AI API or
transport error without inspecting saved state. Optional unanswered questions do not block ordinary
work. Required input, conflicts, unverified host state and failed application remain visible.

A global request never starts project discovery. If both scopes were requested, continue the already
authorized named project after global completion. No new provider API, credential, model selection,
fast-mode setting or model/subagent process is part of this workflow. Read recovery only when the
CLI reports a concrete installation or apply failure.
