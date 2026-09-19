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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:b2d3c7a9a42ce53e2ab8806401efb6e7076d7550843f0a09dd7158de56eee08f"
  - "repo:docs/guides/installed-usage-guard.md#sha256:c94975f1e11052ebf9c04e00066fe121229eea186945c056164d3a33c609df87"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:72bd93df896956e10c30013db3bc9232b7a4a3ca"
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
