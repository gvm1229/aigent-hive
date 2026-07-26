---
name: hive-knowledge-promote
description: Review and promote an eligible project-neutral fact, reusable preference, or portable workflow from project knowledge into user-scope Hive knowledge.
---

# Hive knowledge promotion

Use only when the user identifies or approves an exact project Wiki page.

1. Classify the candidate as exactly one of `fact`, `preference`, or `workflow`.
2. Reject confidential, credential-adjacent, project-exclusive, private-path, unrelated,
   ambiguous, or excluded content.
3. Resolve the user store bound during project setup as `<user-root>`.
4. Preview:
   `hive knowledge promote --target <project-root> --user-root <user-root> --page-id <id> --category <category> --dry-run --output json`.
5. Show redaction, provenance, deduplication, contradiction, and replacement decisions.
6. Apply only after explicit approval:
   `hive knowledge promote --target <project-root> --user-root <user-root> --page-id <id> --category <category> --apply --output json`.
7. Confirm root canonical Markdown activation before root SQLite rebuild.

Never promote Raw content directly. Never copy a project path, credential, token, cookie,
private endpoint, customer datum, or repository-exclusive implementation detail.
