---
schema_version: 1
pair_id: knowledge-storage
topic_slug: knowledge-storage
language: ko
counterpart: ../en/knowledge-storage.md
title: "지식 정본 저장"
summary: "Markdown 정본 유지, 2026-10-05 Notion 정본 후보 폐기."
tags: [knowledge, sqlite]
aliases: ["Markdown SQLite 경계"]
sources:
  - "repo:docs/decisions/ADR-0003-markdown-sqlite-boundary.md#sha256:9834a07f92cb41cb60c697f71aed30f8cc7874e338d51eff5a8a365a515a13e6"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:160bc8bc434f1547e1fb3dad23902b740304323c800b92ece62ccda10a61114e"
links: [docs-wiki-architecture, host-external-integrations, shared-index]
reviewed_revision: "git:e91331b5a498484c8abde81fe0989df1bccf8ea6"
status: active
---

# 지식 정본 저장

- 소스 지식·실행·역할·계획: Git 추적 Markdown·YAML·TOML 정본
- 소비자 지식: 로컬 Markdown 정본. SQLite는 재생성 가능한 검색 색인, 유일한 영구 사본 역할 제외
- 2026-10-05 사용자 결정: Notion 정본 후보 폐기, 과거 0.10 목표 버전의 출시 약속 종료
- 기존 Discord 기능과 별도 결정
