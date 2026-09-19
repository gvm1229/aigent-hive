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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:3a88481c3ede3f60aef3a9d39442a7b4064c231ddaa56a3d560f76f329ec6ed9"
  - "repo:docs/guides/installed-usage-guard.md#sha256:3a6aea1c476fb4efcb45de83ed94d35de51abc94e9476694fc5ebe60d15a9fec"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:0340f8a26d14ebcc134c14e421d605f8022912f8"
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
