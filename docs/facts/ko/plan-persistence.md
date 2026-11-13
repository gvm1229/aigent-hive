---
schema_version: 1
pair_id: plan-persistence
topic_slug: plan-persistence
language: ko
counterpart: ../en/plan-persistence.md
title: "계획 Markdown 정본"
summary: "계획 기본값은 canonical Markdown, session 참조는 간결한 요약·경로."
tags: [documentation, plan, state]
aliases: ["Markdown plan authority"]
sources:
  - "repo:.agents/directives/04-documentation-state.md#sha256:8125c0cdce9daeffb4f2c2b4ad258d4b63a0ff363429bfebb4f4d5852532ff91"
  - "repo:.agents/directives/references/planning-contract.md#sha256:4136a403ed93af42bed844f30e7ed48309ab9ed535c3deb91b1d264dab2fda33"
  - "repo:docs/plans/README.md#sha256:c546623067ecf99a30b8e7aedd1b421487e7bd321f71b41f8ad3b4ef7b2e8851"
  - "repo:harness/template/AGENTS.md.jinja#sha256:fe7f06cc88b566a9ab1ef2919d8ca43830aa963a98208c084c56b5c12f26c864"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# 계획 Markdown 정본

현재 요청의 명시적 opt-out과 별도 durability 의무 부재를 제외한 모든 계획:
적절한 canonical Markdown 기록. Session 출력: 저장 계획의 일대일 복제 금지,
간결한 요약·경로 또는 extensive review용 경로만 제공. 완료 기준: source·consumer
guidance 일치와 projection 시험. 요청 배경: 긴 계획 전문의 session 중복 없는
durable plan authority. `PLAN.md` revision: 단조 증가 정수 변경 횟수. 과거 `1.99` 뒤
`2.00`: 새 계획 세대 아닌 99번째 뒤 100번째 변경 표기.
