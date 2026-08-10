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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:3f107ad6b4ac75f191f2bc6933a60d14e1e194b2ed5f12376e433a8f11761b0c"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ab8cfec03bc6fcfb7d0e55e5c47d5c5bc57fa75adcb1993cd55086f686b56741"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:9ad86748e6144d65c143194538ba95369258ea0b00336f1b81d347fe1ce87245"
  - "repo:docs/plans/active/discord-onboarding-v09.md#sha256:1d7cd9c91104647507ca17fd2aaeb336d5a4637a7250a85cb92e4fa9f5dec109"
  - "repo:docs/plans/active/v0.10.0-notion-candidate.md#sha256:f863a6c59dde7c117e9b4b294cb0974e051ffca5970d830cfa75e50d9799dc4f"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:9ffa22cf14504ba7385135c1f62fdcb19bede32a0925ed72eb23fa8b96359eb5"
  - "repo:schemas/user-setup.schema.json#sha256:e83e5f318a5b6ffcc08cfe0898a2b6138512c6bfb0eea99c6070b134f3712f47"
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
