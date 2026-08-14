---
schema_version: 1
pair_id: host-external-integrations
topic_slug: host-external-integrations
language: ko
counterpart: ../en/host-external-integrations.md
title: "Discord v0.9·Notion v0.10 host integration"
summary: "Discord v0.9은 현지화한 구역형 Markdown 사용량 알림을 실제 형식으로 시험·전송하고, Notion 연결·host OAuth는 첫 v0.10 시험판 보류."
tags: [discord, integration, notion]
aliases: ["Host integration priority"]
sources:
  - "repo:crates/hive-cli/src/discord.rs#sha256:8e46be8e49884c9fbfacee0b17c2588bd637ff08118e4d98465dbc7b45ccba77"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:e444d9d2f20bae53556d206481fd999dd0ac2b496868dd7fdc2c8bc0c1502049"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:73d95136c28b6742d76d7aca0874144808500168a67fc97accdee9bc2b387481"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:9ad86748e6144d65c143194538ba95369258ea0b00336f1b81d347fe1ce87245"
  - "repo:docs/plans/active/discord-onboarding-v09.md#sha256:91a27ed57ddd259ac0a3270ee9242243f0a567bdae3fc756b90f76303c01c037"
  - "repo:docs/plans/active/v0.10.0-notion-candidate.md#sha256:f863a6c59dde7c117e9b4b294cb0974e051ffca5970d830cfa75e50d9799dc4f"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:1fcbb2b9b2db6d57bd40682f80db2a0a916ebbffb3434431038b609b6b743c11"
  - "repo:schemas/user-setup.schema.json#sha256:83427614c5b997a695b9f22c52093d4e2d26892b7eb42fc9873309891d0e81e0"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:4c74e7b82263f85bee21a2272dc865eeb60eaa04"
status: active
---

# Discord v0.9·Notion v0.10 host integration

Discord outbound 알림:

- 선택 인터페이스 언어 하나와 제한된 안전 항목·순서 사용
- 시험·실제 알림 공통 renderer, 시험 고지 구역만 실제 내용 앞에 추가
- 빈 줄로 구분한 Markdown 구역: 이모지·굵은 제목의 사용량·작업 정보·작업 계속 요청
- 밑줄 표기: `0건`
- 실제 중단 알림: 원문 prompt·session ID·절대 경로·credential 없이 project·run 제목·checklist 진행 상태·host·남은 사용량 표시
- 전역 설정: webhook 환경 변수 이름과 재개 가능한 비밀 없는 답변만 기록
- Notion: v0.10 내부 후보 유지, v0.9의 OAuth token·webhook URL·원문 prompt·절대 경로 제외
