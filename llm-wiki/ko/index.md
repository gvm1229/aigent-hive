---
schema_version: 1
pair_id: index
topic_slug: index
language: ko
counterpart: ../en/index.md
title: "Aigent Hive Source Wiki"
summary: "Aigent Hive source workspace의 provider-neutral bilingual knowledge 탐색 안내."
tags: [entrypoint, navigation, source-wiki]
aliases: ["Hive 소스 지식"]
sources:
  - "repo:AGENTS.md#sha256:28626a77b614ca70cd09afdeb8be3d0767e5ca088ab52e942bf2af269d7b9cb2"
  - "repo:hive-source.json#sha256:528b3c6a8f8614a38065144f2de9f3cd527474d5e4ec3f720acd6a27e60f2019"
links: [boundaries, crate-architecture, knowledge, plugin-lifecycle, security-release, skill-routing, source-overview, upgrade, usage-hosts, workflow]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# Aigent Hive Source Wiki

Aigent Hive 자체 개발용 durable knowledge. English 정본 위치: `llm-wiki/en/`. 각 문서의
exact Korean counterpart 위치: `llm-wiki/ko/`. Tracked Markdown이 정본이며 local SQLite
index는 삭제·재구축 가능한 projection.

## 탐색 순서

- 목적·소유권·provider neutrality: `source-overview`, `boundaries`
- 구현 구조: `crate-architecture`, `knowledge`, `skill-routing`
- Host lifecycle: `plugin-lifecycle`, `upgrade`, `usage-hosts`
- 신뢰 경계와 maintainer 실무: `security-release`, `workflow`

이 저장소의 class: Hive source workspace. Source directive, runtime scratch data와 Source
Wiki의 consumer harness 출하 금지.
