---
schema_version: 1
pair_id: source-project-update-summary-skill
topic_slug: source-project-update-summary-skill
language: ko
counterpart: ../en/source-project-update-summary-skill.md
title: "Source 프로젝트 전용 업데이트 요약 Skill"
summary: "Aigent Hive source workspace의 비출하 프로젝트 전용 Skill `update-summary`: 개발자·기여자 전용 변경을 제외한 검증된 한국어 구독자 출시 개선 내역과 안정판 Discord 메시지 정본 작성."
tags: [development, release-notes, skill]
aliases: ["update-summary"]
sources:
  - "repo:.agents/skills/update-summary/SKILL.md#sha256:80944d0655ca4f0c2e2ed8f0264ce1cf2d11447302dfb26ed866fe8076afa470"
  - "repo:docs/archive/plans/foundations/source-update-summary-skill.md#sha256:4c2eb48e174ddacef78f3b1d576db2f703f4807632feac925458128da4dd9039"
links: [public-skill-identity, source-development, v0-9-full-release]
reviewed_revision: "git:f1c89f0998447f3bc53fbe0560521874efc65323"
status: active
---

# Source 프로젝트 전용 업데이트 요약 Skill

`update-summary`: 명시 유지보수자 요청 기반 source workspace 전용 Skill. 검증된 현재·직전
안정판 근거로 한국어 구독자 업데이트 작성. 제품 Skill·`harness/`·release bundle·제품 catalog·consumer
projection 제외. 설치 product·사용자 작업 방식·안전 경계·활용 가능한 기능 이해에 직접 영향을 주는 변경만 포함하며,
출시 설명 형식·CI·검증 기록·저장소 계획·기여자 workflow는 설치 product 변경이 없는 한 제외.
안정판의 정확한 한국어 메시지는 `docs/releases/<version>.subscriber.ko.md`에 저장.
