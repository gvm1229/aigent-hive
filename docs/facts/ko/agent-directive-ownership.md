---
schema_version: 1
pair_id: agent-directive-ownership
topic_slug: agent-directive-ownership
language: ko
counterpart: ../en/agent-directive-ownership.md
title: "Agent 지침 단일 소유권"
summary: "Source·소비자 작업 규칙군의 단일 정본 연결과 크기·경로·투영·중복 예산 검사"
tags: [directives, routing, v0-10]
aliases: ["Directive 최적화"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:20905d49494df815461b4e9ffe6df89ee33ccb774510da2cfa10c98f0508b077"
  - "repo:AGENTS.md#sha256:d8fe84d5fe9bf291465651087a79135880c9b6f17e284e65a4eeb0891d851f2f"
  - "repo:docs/architecture/agent-directive-ownership.md#sha256:53476c7ca8f772d1d2bd956616d3b3f8235282a4c0643784e1a41895333cd2a9"
  - "repo:harness/template/AGENTS.md.jinja#sha256:7fdcd351b7d0624baa68d11bf9e850692c9eeaae13abb003295c8727f621543a"
  - "repo:scripts/check-agent-directives.py#sha256:4c9fe2ff89d0429b76c1e7a36fa2a3c5e9a953f29c592fde8b8199d793ab2332"
links: [agent-autonomous-continuation, artifact-boundaries, historical-project-base-coverage]
reviewed_revision: "git:64125db02505a9a696e870d23fa54feb125b8093"
status: active
---

# Agent 지침 단일 소유권

- Source `AGENTS.md`와 소비자 `AGENTS.md` 투영: 짧은 경로 안내자
- 세부 규칙군: 각각 하나의 정본 지침이 소유
- 생성 진입점: ownership 대장에서 허용한 요약만 유지
- 정적 gate: byte 예산·대상 경로·현재 투영 일치·비허용 정규화 규칙 중복 검사
- 과거 project·user base: byte 불변
- 안정판 게시: 버전명을 포함한 명시 승인 필수
