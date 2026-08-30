---
name: knowledge-transfer
description: (knowledge-transfer) Move the user's existing portable Hive knowledge safely between computers without re-extracting it from source folders.
---

# Transfer Hive Knowledge (`knowledge-transfer`)

Use this Skill only for an existing Hive knowledge base moving between computers. Use
`knowledge-scan` when the user wants new knowledge extracted from a repository or folder.

1. Resolve the installed user root and operating system. Show the selected portable collections,
   exclusions, bundle path, SHA-256, and destination free-space check before creating a bundle.
2. Export canonical Markdown with `hive knowledge export`; never include FTS, vector indexes,
   models, host settings, credentials, or confidential collections.
3. On the destination, verify the exact bundle, run import dry-run, explain conflicts, and apply
   only after the reviewed plan is unchanged. Never overwrite divergent canonical files.
4. Confirm FTS rebuild and a representative knowledge query. This completes the transfer.
5. If the destination user preference enables vector search, ask whether to rebuild vectors now.
   A no answer defers this one rebuild without changing the vector preference or transfer result.

Treat imported project-private collections as detached until explicit destination-project attachment.
Do not use a cloud account, move a bundle automatically, or infer approval for confidential data.
