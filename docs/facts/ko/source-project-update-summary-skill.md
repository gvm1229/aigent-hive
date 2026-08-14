---
schema_version: 1
pair_id: source-project-update-summary-skill
topic_slug: source-project-update-summary-skill
language: ko
counterpart: ../en/source-project-update-summary-skill.md
title: "Source 프로젝트 전용 업데이트 요약 Skill"
summary: "Aigent Hive source workspace의 비출하 프로젝트 전용 Skill `update-summary`: 개발자·기여자 전용 변경을 제외한 검증된 한국어 구독자 출시 개선 내역 작성."
tags: [development, release-notes, skill]
aliases: ["update-summary"]
sources:
  - "repo:.agents/skills/update-summary/SKILL.md#sha256:8504ec054123dc8ea1b36383ab8ca3529c96cc4df6b8f7c948bfd21f09796a46"
  - "repo:docs/plans/active/source-update-summary-skill.md#sha256:4c2eb48e174ddacef78f3b1d576db2f703f4807632feac925458128da4dd9039"
links: [public-skill-identity, source-development, v0-9-full-release]
reviewed_revision: "git:26b949e1cfa5bfe4470693c7a1282100a9cb908e"
status: active
---

# Source 프로젝트 전용 업데이트 요약 Skill

`update-summary`: 명시 유지보수자 요청 기반 source workspace 전용 Skill. 검증된 현재·직전
안정판 근거로 한국어 구독자 업데이트 작성. 제품 Skill·`harness/`·release bundle·제품 catalog·consumer
projection 제외. 설치 product·사용자 작업 방식·안전 경계·활용 가능한 기능 이해에 직접 영향을 주는 변경만 포함하며,
출시 설명 형식·CI·검증 기록·저장소 계획·기여자 workflow는 설치 product 변경이 없는 한 제외.
