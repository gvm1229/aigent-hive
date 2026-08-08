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
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:bf987da6a220df4aa4194f87928626ea8321438671c9d4369c8e097fd272c8ec"
  - "repo:docs/plans/active/discord-notion-onboarding.md#sha256:9a444a6787fa527ab5ea96e09bef31610575e700676bd6562739f3f59f5b2a11"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:907817827a6733dd380aaedea2e7592bc10a7311"
status: active
---

# Discord·Notion host integration

구현 완료 범위: typed Notion backend, SQLite projection engine, capability receipt 검증,
Discord outbound notifier core. 미완료 범위: global setup의 Notion browser OAuth 안내·연결 검증,
Discord webhook 설정·시험 알림, HTML 시각 안내. `DNI-*` 후속 범위: 연결 실패 또는 설정 종료 뒤
비밀 없는 진행 기록 보존과 다음 설정의 `전체 검토`, `선택 항목 검토`, `중단한 단계부터 계속` 제공.
계속 선택 전 완료 답변·연결 receipt 재검증. Notion OAuth token·webhook URL·원문 prompt·절대 경로 보존 0건.
