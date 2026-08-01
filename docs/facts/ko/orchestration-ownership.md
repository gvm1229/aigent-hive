---
schema_version: 1
pair_id: orchestration-ownership
topic_slug: orchestration-ownership
language: ko
counterpart: ../en/orchestration-ownership.md
title: "Orchestration ownership"
summary: "0.8 기존 run의 owner pin 유지, v0.9 새 run의 검증된 host-native capability 기본값."
tags: [orchestration, ownership]
aliases: ["Orchestration owner"]
sources:
  - "repo:docs/decisions/ADR-0004-orchestration-ownership.md#sha256:f58554acf449855ca192ac1219d87019ca7ecc665506366455000bac78f24d87"
  - "repo:docs/decisions/ADR-0015-host-native-skill-composition.md#sha256:06938e887dc4992019718ea51ca0ec55f7bea4a56a647dd12409cd22c9375708"
links: [product-non-goals, skill-routing, v0-9-skill-suite-plan]
reviewed_revision: "git:d28c11908507cd0ae9f79ed0dfb4bcabf345ced2"
status: active
---

# Orchestration ownership

`0.8.x`와 기존 run: ADR-0004의 OMX·OMC·host-native owner pin 유지.
v0.9 새 run: ADR-0015의 검증된 host-native capability 기본값.
OMX·OMC: 명시적 사용자 선택 외부 호환 계층, Hive dependency 아님.
공통 경계: pinned run owner의 silent switch 금지.
