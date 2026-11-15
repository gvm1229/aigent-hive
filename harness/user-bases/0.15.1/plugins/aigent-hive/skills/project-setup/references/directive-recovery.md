# Optional directive recovery

For a request such as "keep this project's instructions after compaction", handle this mode
without re-running unrelated project setup questions. The agent runs the commands; users need
not name a Skill or use the CLI. Source workspaces use the source workflow, not consumer setup.

1. Inspect the supported host and current project hooks. Codex is the first qualification target;
   Claude delivery needs separate acceptance. Antigravity context recovery is unavailable.
2. Select only short, non-confidential canonical Markdown excerpts: language, approval boundaries,
   current plan/status, and path-specific editing rules. Do not copy raw conversation or tool output,
   infer authority, summarize recursively, or include every directive. Host/user priority still applies.
3. With the requested scope, prepare the user-owned root `.hive-context.toml`. Preserve existing entries.
   Its schema is `schema_version = 1`, optional `protected_paths = ["relative/path"]`, and `[[rules]]`
   with `id`, `source` (relative Markdown path), `first_line`, `last_line` (inclusive, starting at 1),
   `digest` (`sha256:` of the exact selected UTF-8 bytes, including original line endings),
   `on = "restore"` or `on = "edit"`, and `paths` for edit rules only. Paths are exact files or
   directory prefixes, not glob patterns. Excerpts remain in their canonical Markdown files.
4. Run `hive policy hooks preview --host <host> --target <project> --output json`.
   Show selected text, protected paths, events, changed settings, digest and cost bounds. The CLI
   rejects invalid, stale or oversized excerpts; never silently truncate or auto-refresh a digest.
5. Obtain approval for that exact preview unless it is already explicitly approved. Then run
   `hive policy hooks apply --host <host> --target <project> --output json --confirm <approval_digest>`.
   Existing setup approval is not approval of newly displayed hooks or protected paths.
6. Run `hive policy hooks status --host <host> --target <project> --output json`. Check
   `context_current` separately from configuration presence and verify actual host delivery.
   Never report a configured file as proof of repeated-compaction adherence.
7. Changed excerpts require review and a new preview. For withdrawal, preview with
   `hive policy hooks remove --host <host> --target <project> --output json`, then repeat with
   `--confirm <approval_digest>` under the user's removal request. Preserve the user's Markdown and
   `.hive-context.toml`. Removal remains available even with a malformed context specification.

Restoration has a 4,096 UTF-8 byte limit including core guidance; token counts are not measured.
Every compaction restores again, even when the source is unchanged. Edit details appear only for
matching paths; unrelated successful edits add no explanation. Without a trustworthy compaction
identity, repeated matching edits may repeat their bounded detail. No extra model call or transcript
reading. Native file protection does not cover arbitrary shell, terminal continuation or MCP writes.
