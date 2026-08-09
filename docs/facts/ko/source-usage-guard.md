---
schema_version: 1
pair_id: source-usage-guard
topic_slug: source-usage-guard
language: ko
counterpart: ../en/source-usage-guard.md
title: "Source session usage guard"
summary: "Source execution boundary 검사를 유지하며 단일 product usage guard로 이관."
tags: [guard, source, usage]
aliases: ["Source quota safeguard"]
sources:
  - "repo:docs/guides/source-usage-guard.md#sha256:3feed99484282ad4265e82d2f831859993f8292b92c8369cb57ee7b7b7c04c9d"
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:97d33e84f001d7e456d24cd95b9712bbe8ef5c9133acd5a6f94ca6395981a066"
links: [automatic-dispatch-guard, source-development, windows-watcher-identity]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
status: active
---

# Source session usage guard

현재 source guard 확인 경계: tool, mutation, external write, push, final answer. 계획된 이관은
이 경계를 product `usage-guard` 하나로 유지하고 사용자 전역 한도와 저장소별 선택 조기 중지
override를 사용. 이관 뒤 source-only Skill·adapter·threshold state 제거. Bypass 조건:
explicit intent와 current session·process binding 유지.
