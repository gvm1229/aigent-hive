---
schema_version: 1
pair_id: agent-directive-ownership
topic_slug: agent-directive-ownership
language: ko
counterpart: ../en/agent-directive-ownership.md
title: "Agent 지침 단일 소유권"
summary: "Source·소비자 작업 규칙군의 단일 정본 연결, 새 version 미지정 개발의 활성 version 귀속, 크기·경로·투영·중복 예산 검사"
tags: [directives, routing, v0-10]
aliases: ["Directive 최적화"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:54f993c4403e7de3612efa12f499bbf1c3a957ee5893875cb0c50204505b1f40"
  - "repo:.agents/directives/04-documentation-state.md#sha256:468031856977484a5c5c39e1af40f4efbe7c953ded7bfc6fa7e4bdabcbc00d82"
  - "repo:AGENTS.md#sha256:214d39d1c6a871fce91f14dd543ebdfd46c7e61a448f4e9e88077355c5c648fa"
  - "repo:docs/architecture/agent-directive-ownership.md#sha256:db0d5c52f75c5ce82354cde25673a5ff701bf99b3458f4f311b7844555b0df05"
  - "repo:harness/template/AGENTS.md.jinja#sha256:fe7f06cc88b566a9ab1ef2919d8ca43830aa963a98208c084c56b5c12f26c864"
  - "repo:scripts/check-agent-directives.py#sha256:b575636bcb279f1839fbd57c0aec697da34cd4cdc4709ca66e2bc6e122442b00"
links: [agent-autonomous-continuation, artifact-boundaries, historical-project-base-coverage]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Agent 지침 단일 소유권

- 소스·소비자 `AGENTS.md`: 짧은 경로 안내, 세부 규칙군별 정본 하나
- 생성 진입점: 허용된 요약만 유지
- 정적 검사: 바이트 예산·대상 경로·현재 투영 일치·비허용 정규화 중복
- 과거 프로젝트·사용자 기준본: 바이트 불변
- 안정판 게시: 버전명을 포함한 명시 승인 필수
- 새 버전 미지정 개발: 활성 계획의 제품 버전·다음 번호 시험판 사용, 임의의 미래 버전 금지
- 지속할 저장소 에이전트 규칙: `.agents/directives/`, 계획 한정 규칙은 소유 계획에 기록. 호스트 개인 메모리에만 보관 금지(관리자, 2026-11-12)
