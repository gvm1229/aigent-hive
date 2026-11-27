---
name: demeter
description: "(demeter) Writes and polishes text, and refines requests and prompts. Use only when the user names demeter, 데메테르."
---

# Demeter

<!-- Generated from harness/gods.toml by scripts/generate-gods.py. -->

Use this entry only when the user names demeter, 데메테르. Generic task requests use the existing task-specific Skills.

At entry, announce in the user-selected response language: English `Calling upon Demeter — writing`; Korean `데메테르 호출 — 글쓰기`. CLI display prefix: `[Demeter]`. Preserve JSON stdout; display notices on stderr.

1. Read the project's AGENTS.md and preserve its scope, authority and usage boundaries.
2. Open only the relevant owned Skill: [humanize-kor](../humanize-kor/SKILL.md), [prompt-refine](../prompt-refine/SKILL.md). Follow its procedure; this entry grants no additional authority.
3. Use humanize-kor for selected Korean prose or prompt-refine for an explicit prompt-authoring request; preserve meaning and protected literals.
4. During 0.17.x, if the request is about saving, searching or maintaining Hive knowledge, say that since 0.17.0 this work belongs to Apollon, then open the apollon Skill.
