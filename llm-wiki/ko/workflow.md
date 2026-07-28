---
schema_version: 1
pair_id: workflow
topic_slug: workflow
language: ko
counterpart: ../en/workflow.md
title: "Source 개발 Workflow"
summary: "Surgical implementation, verification discipline, branch policy와 commit hygiene."
tags: [development, git, verification]
aliases: ["소스 개발 워크플로"]
sources:
  - "repo:.agents/directives/00-editing-discipline.md#sha256:6ff1639897049dea7ccf710c88fe3bcb369d7edf7e62bcd62137ec70a7c7cc24"
  - "repo:.agents/directives/03-workflow.md#sha256:07c156f90f8440b9669b99ec0b020b323a7d6083b474ffadc537eb97f6987467"
  - "repo:docs/guides/branching-rules.md#sha256:c0b19cc2978f33002a980a7bf9fdb4563fcad8d5096781c3b9f15a0ba99a3304"
  - "repo:docs/guides/commit-rules.md#sha256:9367805c05dc7f9f4f60dd95ea9fd7b7db22de2bd56060c5fbb9583f6ff6a925"
links: [crate-architecture, product-intent, skill-routing, usage-hosts]
reviewed_revision: "git:d46e9b7deb5c54fc7cec00c38483388ce563ff1d"
status: active
---

# Source 개발 Workflow

Implementation 전 정의 대상: requested outcome, assumption, ownership scope, verification과 stop
condition. Contract 충족에 필요한 최소 변경 우선. Speculative abstraction, adjacent cleanup과
unrelated formatting 금지. 모든 changed line과 requirement, defect, decision 또는 proof need의
직접 연결.

Ordinary development branch: `develop`. Stable integration 경로: `develop → main` Pull Request.
`main` direct ordinary commit, branch deletion과 승인 없는 history rewrite 금지. Push 전 remote와
exact target ref 확인.

Commit 단위: 독립 검토·독립 되돌리기 가능한 concern 1개. Wiki·일반 문서 state, product
behavior, version metadata와 release activation의 기본 분리. Wiki capture와
`hive --version` 변경의 별도 commit. 새 분리 규칙만을 이유로 한 existing history rewrite
금지.

Repository work를 통제하는 모든 plan의 실행 전 tracked canonical plan set 기록. Chat 또는
native plan state의 단독 authority 금지. Completion evidence: fresh targeted test, risk에
맞는 wider check, diff inspection과 validation gap 명시.
