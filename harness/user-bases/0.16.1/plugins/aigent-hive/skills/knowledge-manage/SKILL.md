---
name: knowledge-manage
description: (knowledge-manage) Keep existing Hive knowledge trustworthy and portable on explicit request. Check it, rebuild its search index, clean it up, share a reviewed fact with other work, or move it between computers. Formerly knowledge-maintain, knowledge-promote, knowledge-transfer.
---

# Manage Knowledge (`knowledge-manage`)

Use the narrowest explicit action on knowledge that already exists. Choose exactly one area,
read only its reference, and follow it. Capturing new knowledge, including a repository scan,
belongs to `knowledge-capture`; finding knowledge belongs to `knowledge-recall`.

## Areas

| Request | Read | Commands |
| --- | --- | --- |
| Check, rebuild the index, rescan, delete, suppress, assign subjects, or maintain vector search | [maintenance](references/maintenance.md) | `hive knowledge lint`, `hive index rebuild`, `hive knowledge scan --apply`, `hive knowledge delete`, `hive knowledge suppress`, `hive knowledge subjects`, `hive knowledge vector` |
| Share a reviewed fact, preference, or workflow from one project with the user's other work | [promotion](references/promote.md) | `hive knowledge promote` |
| Move or merge existing portable knowledge between computers | [transfer](references/transfer.md) | `hive knowledge transfer` |

The former names select areas of this Skill: `knowledge-maintain` is maintenance,
`knowledge-promote` is promotion, and `knowledge-transfer` is transfer. When the user names a
former name, say the new name `knowledge-manage` and its area once, then continue.

## Shared rules

- Route by target class first. A valid `hive-source.json` selects `hive source-wiki`; never use
  consumer knowledge paths in the source workspace. Use the project target only for an enabled
  registered project; an unregistered project uses the user root as both `--target` and
  `--user-root`.
- Require explicit intent for each mutation. Lint or rebuild intent never authorizes deletion,
  suppression, promotion, or transfer; show the exact preview and apply only after approval.
- Never promote Raw content directly, promote a scan claim automatically, or trigger promotion
  from retrieval. Automatic promotion belongs only to reviewed scan, rescan, and maintenance apply
  flows.
- Never include confidential knowledge, credentials, FTS or vector indexes, models, or host
  settings in a transfer, and never infer approval for confidential data.
- Keep Markdown canonical and SQLite disposable. Preserve non-Hive files and all user-authored
  bytes outside declared Hive ownership. Report changed paths, evidence digests, and recovery
  guidance from the CLI result.
