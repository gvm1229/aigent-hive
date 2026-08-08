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
  - "repo:crates/hive-cli/src/discord.rs#sha256:524edb6f4e70a64cef99ffdef0e8347275701836c49e5b8b155edd37242fa6bd"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:ad1c22fbacbfab22bd4120a94bad5cb10ebc45936e39c5b0f586f5d9a2467a92"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:9d0a796027dde450cfec2162ac1073305068aa4ac0e6351303f4976c1ad87f38"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:9ad86748e6144d65c143194538ba95369258ea0b00336f1b81d347fe1ce87245"
  - "repo:docs/plans/active/discord-onboarding-v09.md#sha256:1ef1d3b61747f317ae1e7fced6e5dd60a6a1a09a6295fcb793d52634ba4098e9"
  - "repo:docs/plans/active/v0.10.0-notion-candidate.md#sha256:f863a6c59dde7c117e9b4b294cb0974e051ffca5970d830cfa75e50d9799dc4f"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
  - "repo:harness/skills/configure/SKILL.md#sha256:abeb032e21d2576366025465d54080966767fb7e17cca57848acf093eaa83eaf"
  - "repo:schemas/user-setup.schema.json#sha256:46b360a9f91e154d1440e2997b56a964edd122383ccfc9b105b4e2ae4f8939f9"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:31f5c7616a14d63a68aee677a5b242ff5c5054e8"
status: active
---

# Discord v0.9·Notion v0.10 host integration

Discord outbound 알림은 제한된 안전 항목을 원하는 순서로 고르고, 선택한 인터페이스 언어 하나만
사용한다. 시험 알림은 실제 알림과 같은 renderer·항목·순서를 사용하며, 형식 변경을 요청할 수 있다는
현지화된 첫 줄만 추가한다. 기본값에는 남은 사용량과 안전한 프로젝트 식별자가 포함된다. 요청 내용은
이 컴퓨터에 유지하고 정본 진행 상태는 `DIS9-005–006` 완료 전까지 확인 불가로 표시한다. Notion은
v0.10 내부 후보로 남으며 v0.9은 OAuth token·webhook URL·원문 prompt·절대 경로를 제외한다.
