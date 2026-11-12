---
name: demeter
description: "(demeter) knowledge; 지식. Names: demeter, 데메테르, demeter. Use for recall project knowledge; find a saved fact; search reviewed knowledge; 프로젝트 지식 조회; 저장된 사실 찾기; 검토 지식 검색."
---

# Demeter

Use this entry when the user names demeter, 데메테르, or when the task matches the description. Host-native Skill selection may select it implicitly; no classifier hook.

At entry, announce in the user-selected response language: English `Calling upon Demeter — knowledge`; Korean `데메테르 호출 — 지식`. CLI display prefix: `[Demeter]`. Preserve JSON stdout; display notices on stderr.

1. Read the project's AGENTS.md and preserve its scope, authority and usage boundaries.
2. Open [the existing knowledge-recall Skill](../knowledge-recall/SKILL.md) and follow its procedure. This entry grants no additional mutation, delegation or publication authority.
3. For cleanup requests, inspect exact candidates first and follow the existing storage/prune preview and explicit removal approval rules. A request to plan cleanup grants no deletion authority.
4. For implementation requests, follow the repository's saved implementation plan and checks. Do not treat a code-polish connection as permission for unrelated edits.
5. Keep model calls and process launch host-owned. New orchestration stays default-off until qualified host receipts, cancellation and reclaim exist.
