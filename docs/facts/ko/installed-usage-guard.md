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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:267309f9d7f56095b6cc00f4602aeaf1249e0e8988f7fc5bd897dff04e2be5f5"
  - "repo:docs/guides/installed-usage-guard.md#sha256:9cf01b711909bca15472e95b5a25325b7094df4115795b6fdf04cf2bc3f017f5"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:4afd5ba4483d98f63ae42c065837f9b010506657"
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
