---
schema_version: 1
pair_id: host-external-integrations
topic_slug: host-external-integrations
language: ko
counterpart: ../en/host-external-integrations.md
title: "Discord v0.9·Notion v0.10 host integration"
summary: "Discord v0.9은 현지화한 outbound 사용량 알림 항목을 안전하게 선택하고 실제 형식을 시험할 수 있으며, Notion 연결·host OAuth는 첫 v0.10 시험판 보류."
tags: [discord, integration, notion]
aliases: ["Host integration priority"]
sources:
  - "repo:crates/hive-cli/src/discord.rs#sha256:8084b804ff091920b2ed588c04d0fce46e617196f78a93ec2bdb01e358a0489c"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:3f107ad6b4ac75f191f2bc6933a60d14e1e194b2ed5f12376e433a8f11761b0c"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ab8cfec03bc6fcfb7d0e55e5c47d5c5bc57fa75adcb1993cd55086f686b56741"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:9ad86748e6144d65c143194538ba95369258ea0b00336f1b81d347fe1ce87245"
  - "repo:docs/plans/active/discord-onboarding-v09.md#sha256:74902022f12fcd031e58603c5b1867268833e993dbf7624a5a3123a42b8c9d6f"
  - "repo:docs/plans/active/v0.10.0-notion-candidate.md#sha256:f863a6c59dde7c117e9b4b294cb0974e051ffca5970d830cfa75e50d9799dc4f"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:9ffa22cf14504ba7385135c1f62fdcb19bede32a0925ed72eb23fa8b96359eb5"
  - "repo:schemas/user-setup.schema.json#sha256:e83e5f318a5b6ffcc08cfe0898a2b6138512c6bfb0eea99c6070b134f3712f47"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:31f5c7616a14d63a68aee677a5b242ff5c5054e8"
status: active
---

# Discord v0.9·Notion v0.10 host integration

Discord outbound 알림은 제한된 안전 항목을 원하는 순서로 고르고, 선택한 인터페이스 언어 하나만
사용한다. 시험 알림은 실제 알림과 같은 renderer·항목·순서를 사용하며, 형식 변경을 요청할 수 있다는
현지화된 첫 줄만 추가한다. 실제 중단 알림은 원문 prompt·session ID·절대 경로·credential 없이 안전한
프로젝트·run 제목·checklist 진행 상태·host·남은 사용량을 표시한다. 전역 설정은 webhook 환경 변수
이름과 재개 가능한 비밀 없는 답변만 기록. Notion은 v0.10 내부 후보로 남으며 v0.9은 OAuth token·webhook
URL·원문 prompt·절대 경로를 제외한다.
