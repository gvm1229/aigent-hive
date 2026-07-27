---
schema_version: 1
pair_id: boundaries
topic_slug: boundaries
language: ko
counterpart: ../en/boundaries.md
title: "소유권과 Orchestration 경계"
summary: "Source, release, consumer 소유권과 replaceable orchestration dependency의 분리."
tags: [boundaries, orchestration, ownership]
aliases: ["Hive 소유권 경계"]
sources:
  - "repo:AGENTS.md#sha256:28626a77b614ca70cd09afdeb8be3d0767e5ca088ab52e942bf2af269d7b9cb2"
  - "repo:docs/decisions/ADR-0001-source-release-installed-boundary.md#sha256:51850d51887f4d2cd4759e562aedee458398463e2b219cb94ca7b4540ad5bab7"
  - "repo:docs/decisions/ADR-0011-source-wiki-independence.md#sha256:e5315d16b0dc932bcedc79add82460220c64bec84e5f1e30e2ed672c93eaa5d4"
links: [knowledge, plugin-lifecycle, source-overview]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# 소유권과 Orchestration 경계

Hive artifact class 세 가지: source workspace, immutable release bundle, installed consumer
harness. Source directive의 consumer 출하 금지. Installed user state, runtime data와 project
knowledge의 source 역수입 금지. Hive mutation 권한: manifest가 선언한 owned path 또는 exact
owned marker block. User-authored·third-party bytes 보존.

## OMX Wiki Skill 제외 이유

- 현재 역할: Codex의 OMX와 Claude의 OMC를 source 개발 orchestration 보조로 적극 활용
- 제외 근거: 고정 `omx_wiki/` 저장 경로, `omx wiki` 명령 surface와
  `.omx-config.json` lifecycle을 통한 durable Hive-owned knowledge와 replaceable tooling의
  결합
- 소유권: Hive의 provider-neutral `llm-wiki/` 계약에 canonical source knowledge 유지,
  SQLite는 derived local index로만 사용
- 향후 retirement 조건: OMX/OMC 도구 교체만 필요, source knowledge migration 0건

이 판단은 OMX Wiki의 품질·유용성 평가와 무관. 실행 보조에는 OMX/OMC 활용, durable data,
path, schema와 Skill identity에는 해당 namespace 비결합.
