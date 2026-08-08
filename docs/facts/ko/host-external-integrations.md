---
schema_version: 1
pair_id: host-external-integrations
topic_slug: host-external-integrations
language: ko
counterpart: ../en/host-external-integrations.md
title: "Discord v0.9·Notion v0.10 host integration"
summary: "Discord는 v0.9 설정 목표 유지, Notion 연결·host OAuth는 첫 v0.10 시험판 보류."
tags: [discord, integration, notion]
aliases: ["Host integration priority"]
sources:
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:9ad86748e6144d65c143194538ba95369258ea0b00336f1b81d347fe1ce87245"
  - "repo:docs/plans/active/discord-onboarding-v09.md#sha256:931d25f2b4d2673fbce477b5c0972bb54fe9b604b12f07ee541b92389dfcb6e8"
  - "repo:docs/plans/active/v0.10.0-notion-candidate.md#sha256:f863a6c59dde7c117e9b4b294cb0974e051ffca5970d830cfa75e50d9799dc4f"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:2b819c1060972bb2416a751ff17e596094b00a6b"
status: active
---

# Discord v0.9·Notion v0.10 host integration

구현 완료 범위: Discord outbound notifier core와 재개 가능한 setup primitive. Discord webhook 설정·
project-aware 알림·HTML 안내: `DIS9-*` 후속 범위. typed Notion backend·SQLite projection engine·
capability receipt 검증: 내부 v0.10 후보. v0.9 setup·help·README·release note의 공개 범위 제외.
Notion OAuth token·webhook URL·원문 prompt·절대 경로 보존 0건.
