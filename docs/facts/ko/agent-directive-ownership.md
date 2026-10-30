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
  - "repo:AGENTS.md#sha256:8b785024ed26e2d916e39dc4b96c90c64171c8eb28f671c4c7f8af1de77d0a68"
  - "repo:docs/architecture/agent-directive-ownership.md#sha256:db0d5c52f75c5ce82354cde25673a5ff701bf99b3458f4f311b7844555b0df05"
  - "repo:harness/template/AGENTS.md.jinja#sha256:ab0902d90e75ac32219217b3617c46008f3a906a84a68453bf389fd69e9ec529"
  - "repo:scripts/check-agent-directives.py#sha256:d0520ff0a53524541cff7965258c89e2a2a2d03df1066312d854ac1ced41e5f9"
links: [agent-autonomous-continuation, artifact-boundaries, historical-project-base-coverage]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Agent 지침 단일 소유권

- Source `AGENTS.md`와 소비자 `AGENTS.md` 투영: 짧은 경로 안내자
- 세부 규칙군: 각각 하나의 정본 지침이 소유
- 생성 진입점: ownership 대장에서 허용한 요약만 유지
- 정적 gate: byte 예산·대상 경로·현재 투영 일치·비허용 정규화 규칙 중복 검사
- 과거 project·user base: byte 불변
- 안정판 게시: 버전명을 포함한 명시 승인 필수
- 새 version 미지정 개발 요청: 활성 계획의 product version·다음 번호 시험판에 귀속
- 임의의 미래 version 제안·이전 금지
