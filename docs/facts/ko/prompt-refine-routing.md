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
  - "repo:docs/decisions/ADR-0009-user-plugin-project-knowledge-boundary.md#sha256:59129f4216306b3c095ab64574700135da0f289df4aab6554f0213e24c40c6f3"
  - "repo:docs/plans/active/prompt-refine-auto-routing.md#sha256:2c70a7ef894396d4bf1a3160c59d42f81e60c80905fe7c00d5f19f33de411b03"
links: [orchestration-ownership, skill-routing]
reviewed_revision: "git:507cdf98de2b0873b0e554fd1bc53810b11c7dc0"
status: active
---

# Prompt refine 승인 routing

명시적 prompt 작성과 material ambiguity가 있는 ordinary work:
`hive-prompt-refine`의 `refine-only` route. Refined prompt·digest 반환 상태:
`awaiting-approval`. 승인 전 project read·tool·write·memory capture·run 생성·task
execution 0건. 실행 권한: explicit `--run` 또는 exact digest에 결합된 후속 승인.
제외: simple question, editless question, clear work, prompt-classifier hook.
