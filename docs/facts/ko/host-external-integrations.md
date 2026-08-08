---
schema_version: 1
pair_id: host-external-integrations
topic_slug: host-external-integrations
language: ko
counterpart: ../en/host-external-integrations.md
title: "Discord·Notion host integration"
summary: "Integration core 구현 완료, 재개 가능한 end-to-end global setup·host OAuth handoff·project-aware Discord 알림은 후속 계획."
tags: [discord, integration, notion]
aliases: ["Host integration priority"]
sources:
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:2e268d2a33c699c6b77a5c711df6a50eaf95624964dc616848bf29321de3624d"
  - "repo:docs/plans/active/discord-notion-onboarding.md#sha256:0b5eb5f5b735c6c29cefe5c6fa1d034f32ffd6d4c6bc4ea45bcde86ed0e43702"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:2b819c1060972bb2416a751ff17e596094b00a6b"
status: active
---

# Discord·Notion host integration

구현 완료 범위: typed Notion backend, SQLite projection engine, capability receipt 검증,
Discord outbound notifier core. 미완료 범위: global setup의 Notion browser OAuth 안내·연결 검증,
Discord webhook 설정·시험 알림, HTML 시각 안내. `DNI-*` 후속 범위: 연결 실패 또는 설정 종료 뒤
비밀 없는 진행 기록 보존과 다음 설정의 `전체 검토`, `선택 항목 검토`, `중단한 단계부터 계속` 제공.
계속 선택 전 완료 답변·연결 receipt 재검증. Notion OAuth token·webhook URL·원문 prompt·절대 경로 보존 0건.
