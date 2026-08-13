---
schema_version: 1
pair_id: installed-usage-guard
topic_slug: installed-usage-guard
language: ko
counterpart: ../en/installed-usage-guard.md
title: "설치 guard 대상 경계"
summary: "설정 완료 Hive project와 Hive source에만 설치 guard 적용, non-Hive folder 전체 비활성."
tags: [guard, source, usage]
aliases: ["Installed usage policy"]
sources:
  - "repo:.github/workflows/ci.yml#sha256:bcba0d0f834f9e1e0dca81f465bb0337c5c4db83299c25d357f132f5a4cefd4d"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:e444d9d2f20bae53556d206481fd999dd0ac2b496868dd7fdc2c8bc0c1502049"
  - "repo:docs/guides/installed-usage-guard.md#sha256:3224f7e04c9025cd788e14506295a723f1d87c97d59f9e629dcfe9bddcb1a302"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:39569b7a2a7c67f8ab19010db8c4df32da470f86"
status: active
---

# 설치 guard 대상 경계

- 단일 구현: 설치 product `usage-guard`
- 설정 완료 Hive project: `max(global, project)`, project-local session state 허용
- Aigent Hive source: global threshold와 user-root runtime, source `.hive/` 생성 `0건`
- 자체 `AGENTS.md`만 보유한 folder·빈 folder: enforce·threshold mutation·session override·halt·runtime `0건`
- Non-Hive setup-free Skill: 사용 가능
- Session control: explicit configured target 사용, 무관한 malformed graph `CURRENT.md` 보존·authority 미사용
- Source task: 시작 preflight 1회, Python watcher·tool 경계 반복 gate·삭제 test의 CI 호출 `0건`
