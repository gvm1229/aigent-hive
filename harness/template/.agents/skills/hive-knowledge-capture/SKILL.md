---
name: hive-knowledge-capture
description: Integrate an explicitly selected source or an agent-reviewed durable task fact into canonical Hive knowledge through the signed Hive CLI. Use for explicit capture requests and automatically at the completion gate when Wiki is enabled and material work produced reusable knowledge; never ingest raw transcripts or unreviewed secret-bearing content.
---

# Hive Knowledge Capture

Capture only user-selected or current-task-authorized, Git-suitable project knowledge.

## Workflow

1. Confirm the source is explicitly selected or was created/revised by the current authorized task,
   no larger than the CLI limit, non-confidential, and suitable for tracking.
2. Prepare or review a Wiki Markdown draft that follows the installed `.hive/knowledge/Schema` contract.
   For agent-reviewed task-fact autocapture, include the bounded outcome, tool or project, creation
   or acceptance criteria, and originating request summary. Preserve exact request text only when
   the user explicitly requests retention and the text passes safety review.
3. Run:

   ```text
   hive knowledge ingest --target <project-root> --user-root <user-root> --source <source-file> --wiki <reviewed-wiki-draft> --output json
   ```

4. Require a schema-valid success result and report its changed paths and evidence digest.
5. Run `hive knowledge lint --target <project-root> --user-root <user-root> --output json`.

## Safety

- Do not capture when Wiki is disabled.
- Never ingest a raw transcript, complete conversation, hook payload, tool output, hidden prompt,
  cache, database, or runtime state.
- Never ingest secrets, provider credentials, caches, databases, or unbounded files.
- Keep Raw and Wiki Markdown canonical; treat SQLite as disposable derived state.
- Do not reproduce CLI mutation logic or write knowledge files directly when the command is unavailable.
