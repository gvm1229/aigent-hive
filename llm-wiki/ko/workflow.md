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
  - "repo:docs/guides/branching-rules.md#sha256:c0b19cc2978f33002a980a7bf9fdb4563fcad8d5096781c3b9f15a0ba99a3304"
  - "repo:docs/guides/commit-rules.md#sha256:443986db38ba26db52106b49ef92d741b103f5b73f82d95e24f8bfcc20ed2887"
links: [crate-architecture, skill-routing, usage-hosts]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
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

Commit 단위: clear concern 1개와 intended file만 stage. 제목: 간결한 한국어 Conventional
Commit, automated co-author trailer 금지. Completion evidence: fresh targeted test, risk에 맞는
wider lint·build, diff inspection과 validation gap 명시.
