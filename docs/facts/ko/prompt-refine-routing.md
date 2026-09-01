---
schema_version: 1
pair_id: prompt-refine-routing
topic_slug: prompt-refine-routing
language: ko
counterpart: ../en/prompt-refine-routing.md
title: "Prompt refine 승인 routing"
summary: "Material ambiguity가 있는 work의 refine-only 진입과 exact 사용자 승인 대기."
tags: [prompt, routing, skill]
aliases: ["Prompt approval gate"]
sources:
  - "repo:docs/archive/plans/foundations/prompt-refine-auto-routing.md#sha256:a56c022be4e24ac6e7acf402e186d1ddbe4a1a39bc4d2c0eb16104e472b3108a"
  - "repo:docs/decisions/ADR-0009-user-plugin-project-knowledge-boundary.md#sha256:a78f7c3acbe764bc04916912e9fbb15bd9c5b90275db7f376add543439f1e90a"
links: [orchestration-ownership, skill-routing]
reviewed_revision: "git:bf7c1d3e36cd94e8ee5f2a68d9f8ca5c4c9f9c87"
status: active
---

# Prompt refine 승인 routing

명시적 prompt 작성과 material ambiguity가 있는 ordinary work:
`prompt-refine`의 `refine-only` route. Refined prompt·digest 반환 상태:
`awaiting-approval`. 승인 전 project read·tool·write·memory capture·run 생성·task
execution 0건. 실행 권한: explicit `--run` 또는 exact digest에 결합된 후속 승인.
제외: simple question, editless question, clear work, prompt-classifier hook.
