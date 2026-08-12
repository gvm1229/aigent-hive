---
schema_version: 1
pair_id: source-usage-guard
topic_slug: source-usage-guard
language: ko
counterpart: ../en/source-usage-guard.md
title: "Source session usage guard"
summary: "저장소 source gate와 단일 product usage guard로 source execution boundary 검사 유지."
tags: [guard, source, usage]
aliases: ["Source quota safeguard"]
sources:
  - "repo:docs/guides/source-usage-guard.md#sha256:5f3fb38548cc8c96cdf9cfe273b77dd4b11c3bea4e0d379c1fefdf40193a0213"
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:7b64cee13b39806a519ee9d8387972a1e69da108e1075b8b0b873581d46c439b"
links: [automatic-dispatch-guard, source-development, windows-watcher-identity]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
status: active
---

# Source session usage guard

저장소 source guard 확인 경계: tool, mutation, external write, push, final answer. 사용자용
제어: product `usage-guard` 하나. 사용자 전역 한도와 저장소별 선택 조기 중지 override 사용.
source-only Skill·adapter·threshold state: `0건`. Bypass 조건: explicit intent와 current
session·process binding 유지.
