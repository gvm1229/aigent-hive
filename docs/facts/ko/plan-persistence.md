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
  - "repo:.agents/directives/04-documentation-state.md#sha256:c2ee469c7fd392a28f63dc5cb545d62d5ed2f794c63b12d766bdef06ed6c650c"
  - "repo:docs/plans/README.md#sha256:85944730779c8686d4f436fe735f8e65b0ee34f8e5dee048103a8e85cd3f508a"
  - "repo:harness/template/AGENTS.md.jinja#sha256:b11663ebd662eb679c11cf115223c5eb47ab762ccd1c26966e83d594b403b67b"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:838842805e453e0508d054e4aa67d7a59b3aa53f"
status: active
---

# 계획 Markdown 정본

현재 요청의 명시적 opt-out과 별도 durability 의무 부재를 제외한 모든 계획:
적절한 canonical Markdown 기록. Session 출력: 저장 계획의 일대일 복제 금지,
간결한 요약·경로 또는 extensive review용 경로만 제공. 완료 기준: source·consumer
guidance 일치와 projection 시험. 요청 배경: 긴 계획 전문의 session 중복 없는
durable plan authority. `PLAN.md` revision: 단조 증가 정수 변경 횟수. 과거 `1.99` 뒤
`2.00`: 새 계획 세대 아닌 99번째 뒤 100번째 변경 표기.
