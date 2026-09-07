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
  - "repo:.agents/directives/01-behavior.md#sha256:7679dd5603fdfa1104b0017e9fe7c7acb6a81f9e4095233ccddf3db01a325af8"
  - "repo:AGENTS.md#sha256:127f2532d044071d58a1d28dc351aba100029f59513a5b45d35f9a112382d6b3"
  - "repo:docs/architecture/agent-directive-ownership.md#sha256:9f845cd258a9e88a0f60fca409cd53c13c6f714ad706d4ba41287c79837197b9"
  - "repo:harness/template/AGENTS.md.jinja#sha256:27a80d0856d5f69ed2670eb441068950a42b207d592d9ba892542aa56c99bffc"
  - "repo:scripts/check-agent-directives.py#sha256:4c9fe2ff89d0429b76c1e7a36fa2a3c5e9a953f29c592fde8b8199d793ab2332"
links: [agent-autonomous-continuation, artifact-boundaries, historical-project-base-coverage]
reviewed_revision: "git:f34c524da540a97d6c2810fb1d0b092bbf1421ed"
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
