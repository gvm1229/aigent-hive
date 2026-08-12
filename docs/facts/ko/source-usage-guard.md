---
schema_version: 1
pair_id: source-usage-guard
topic_slug: source-usage-guard
language: ko
counterpart: ../en/source-usage-guard.md
title: "설치 guard 대상 경계"
summary: "설정 완료 Hive project와 Hive source에만 설치 guard 적용, non-Hive folder 전체 비활성."
tags: [guard, source, usage]
aliases: ["Source quota safeguard"]
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:b5072a86240d87679c63a28568e1af75928367d037ddaa6af4cb9210bc4c2880"
  - "repo:docs/guides/source-usage-guard.md#sha256:c4c7f5f717627becc0636d1c7320eb227df844dc2ea5d837a79080c07c673197"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:ced55f4d0b18b259c9b43e0f9622b6d617a65737"
status: active
---

# 설치 guard 대상 경계

- 단일 구현: 설치 product `usage-guard`
- 설정 완료 Hive project: `max(global, project)`, project-local session state 허용
- Aigent Hive source: global threshold와 user-root runtime, source `.hive/` 생성 `0건`
- 자체 `AGENTS.md`만 보유한 folder·빈 folder: enforce·threshold mutation·session override·halt·runtime `0건`
- Non-Hive setup-free Skill: 사용 가능
- Source task: 시작 preflight 1회, Python watcher·tool 경계 반복 gate `0건`
