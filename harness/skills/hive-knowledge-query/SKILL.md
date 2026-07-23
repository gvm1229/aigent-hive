---
name: hive-knowledge-query
description: Query canonical Hive knowledge by text or tag without changing canonical project data. Use when an answer explicitly requires approved project memory or Wiki facts; do not use for simple questions that need no project context.
---

# Hive Knowledge Query

Query only the installed project's canonical knowledge projection.

## Workflow

1. Keep the simple-question path isolated; use this Skill only when project knowledge is required.
2. Run exactly one bounded query:

   ```text
   hive knowledge query --target <project-root> --text <query> --limit <1..100> --output json
   ```

   Or use `--tag <tag>` instead of `--text <query>`.
3. Cite the returned canonical locators and distinguish retrieved facts from inference.
4. If the derived index is stale or unavailable, run no write automatically. Report that `hive index rebuild` is the explicit maintenance action.

## Safety

- Do not ingest, suppress, delete, or rewrite knowledge.
- Do not search provider credentials, runtime state, caches, or unrelated project files.
- Do not treat SQLite-only content as canonical.
