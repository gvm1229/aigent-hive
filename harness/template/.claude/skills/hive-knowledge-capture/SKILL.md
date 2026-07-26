---
name: hive-knowledge-capture
description: Integrate an explicitly selected non-confidential source and reviewed Wiki draft into canonical Hive knowledge through the signed Hive CLI. Use when the user asks to capture, ingest, or add durable project knowledge; do not use for automatic memory ingestion or unreviewed secret-bearing content.
---

# Hive Knowledge Capture

Capture only user-selected, Git-suitable project knowledge.

## Workflow

1. Confirm the source is explicitly in scope, no larger than the CLI limit, non-confidential, and suitable for tracking.
2. Prepare or review a Wiki Markdown draft that follows the installed `.hive/knowledge/Schema` contract.
3. Run:

   ```text
   hive knowledge ingest --target <project-root> --source <source-file> --wiki <reviewed-wiki-draft> --output json
   ```

4. Require a schema-valid success result and report its changed paths and evidence digest.
5. Run `hive knowledge lint --target <project-root> --output json`.

## Safety

- Never ingest automatically from ordinary conversation, hooks, tool output, transcripts, or runtime state.
- Never ingest secrets, provider credentials, caches, databases, or unbounded files.
- Keep Raw and Wiki Markdown canonical; treat SQLite as disposable derived state.
- Do not reproduce CLI mutation logic or write knowledge files directly when the command is unavailable.
