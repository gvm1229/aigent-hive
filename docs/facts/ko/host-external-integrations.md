---
schema_version: 1
pair_id: host-external-integrations
topic_slug: host-external-integrations
language: ko
counterpart: ../en/host-external-integrations.md
title: "Discord·Notion host integration"
summary: "Integration core 구현 완료, end-to-end global setup·host OAuth handoff·project-aware Discord 알림은 후속 계획."
tags: [discord, integration, notion]
aliases: ["Host integration priority"]
sources:
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:4d6b1e5a018e5ef5ed129323927c191c1d74208a8c3d2d5b05678096629e9f82"
  - "repo:docs/plans/active/discord-notion-onboarding.md#sha256:acdd99039a9b030af94549ba7fb7eb9c9fbf6d51002ba0584082f0d623a3c6dc"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:b97e5fdeff0be50747d147dad8f8b8c2dcc8487f0e54ea28decbc0da30cecf08"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:a8f2ef61565e15edef9e42355877f2d393058f80"
status: active
---

# Discord·Notion host integration

구현 완료 범위: typed Notion backend, SQLite projection engine, capability receipt 검증,
Discord outbound notifier core. 미완료 범위: global setup의 Notion 선택·host-owned browser
OAuth 연결 검증, Discord 설정·시험 알림, HTML 시각 안내. `DNI-*` 후속 범위:
안전한 project identity·run·요청 요약·canonical 진행 상태·checkpoint·재개 안내를
usage-guard 알림에 결합. 원문 prompt 기본 제외, 명시적 opt-in·preview·redaction 필수.
