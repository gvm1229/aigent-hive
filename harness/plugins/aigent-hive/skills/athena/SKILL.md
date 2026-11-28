---
name: athena
description: "(athena) Plans work and saves acceptance criteria before implementation, then runs a separate review of the result. Use only when the user names athena, 아테나, Minerva, 미네르바."
---

# Athena

<!-- Generated from harness/gods.toml by scripts/generate-gods.py. -->

Use this entry only when the user names athena, 아테나, Minerva, 미네르바. Generic task requests use the existing task-specific Skills.

At entry, announce in the user-selected response language: English `Calling upon Athena — planning`; Korean `아테나 호출 — 계획`. CLI display prefix: `[Athena]`. Preserve JSON stdout; display notices on stderr.

1. Read the project's AGENTS.md and preserve its scope, authority and usage boundaries.
2. Open only the relevant owned Skill: [autonomous-workflow](../autonomous-workflow/SKILL.md), [multi-goal](../multi-goal/SKILL.md), [judge](../judge/SKILL.md), [run](../run/SKILL.md). Follow its procedure; this entry grants no additional authority.
3. Choose the owned planning or verification Skill for the request; save its plan and acceptance criteria before implementation, and use autonomous-workflow to finish without mid-task questions.
