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
  - "repo:.agents/directives/04-documentation-state.md#sha256:5660c7d72b0bb89f8d105a50d7d3768bcf93d3728d855704df5bfad815744d02"
  - "repo:docs/plans/PLAN.md#sha256:01ad88238def1785e8b9fcc04d9bb89042be86323341905f8b50c14c5d10c8ab"
  - "repo:docs/plans/README.md#sha256:7fca19e770b1b99b647a893517b50bcf6e6eb136e3c84ae52ba1258267087df0"
  - "repo:harness/template/AGENTS.md.jinja#sha256:9b45162b48b5d6eedcce0d0b6bc2d63613776fc1089a7d2878941897d0d506c5"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:a833ba22d805fedce23cd74aa326b574b77280f4"
status: active
---

# 계획 Markdown 정본

현재 요청의 명시적 opt-out과 별도 durability 의무 부재를 제외한 모든 계획:
적절한 canonical Markdown 기록. Session 출력: 저장 계획의 일대일 복제 금지,
간결한 요약·경로 또는 extensive review용 경로만 제공. 완료 기준: source·consumer
guidance 일치와 projection 시험. 요청 배경: 긴 계획 전문의 session 중복 없는
durable plan authority. `PLAN.md` revision: 단조 증가 정수 변경 횟수. 과거 `1.99` 뒤
`2.00`: 새 계획 세대 아닌 99번째 뒤 100번째 변경 표기. 현재 정규화 값: `185`.
