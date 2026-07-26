---
name: hive-knowledge-query
description: Query canonical Hive knowledge by text or tag without changing canonical project data. Use when an answer explicitly requires approved project memory or Wiki facts; do not use for simple questions that need no project context.
---

# Hive Knowledge Query

Query the installed project's canonical knowledge first, then the explicitly bound user-root
knowledge projection.

## Workflow

1. Keep the simple-question path isolated; use this Skill only when project knowledge is required.
2. Resolve the user store bound during project setup as `<user-root>`.
3. Run exactly one bounded combined query:

   ```text
   hive knowledge query --target <project-root> --user-root <user-root> --text <query> --limit <1..100> --output json
   ```

   Or use `--tag <tag>` instead of `--text <query>`.
4. Preserve `project-first` precedence. Cite each returned canonical locator, `scope`, and
   promotion provenance; distinguish retrieved facts from inference.
5. If either derived index is stale or unavailable, run no write automatically. Report that
   `hive index rebuild --target <project-root>` or
   `hive index rebuild --target <user-root>` is the explicit maintenance action.

## Safety

- Do not ingest, suppress, delete, or rewrite knowledge.
- Do not search provider credentials, runtime state, caches, or unrelated project files.
- Do not treat SQLite-only content as canonical.
