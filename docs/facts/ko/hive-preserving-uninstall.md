---
schema_version: 1
pair_id: hive-preserving-uninstall
topic_slug: hive-preserving-uninstall
language: ko
counterpart: ../en/hive-preserving-uninstall.md
title: "Hive 보존형 제거"
summary: "hive uninstall: Hive가 추가한 사용자 범위 설정만 제거, knowledge base·저장 user preference 항상 보존."
tags: [bootstrap, onboarding, preservation, uninstall]
aliases: ["clean reinstall", "hive uninstall"]
sources:
  - "repo:crates/hive-cli/src/user_install.rs#sha256:8a2fc46f070b2d093b5826cf59b19a718a6ffbcb71d1516212f1b9a584f42c50"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:0bfd9117a0d835da5f19bc02b82959a5630a4955a81ee3efda0a6ba5246dfaad"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:4f3676378fafac75f9c6376210c760a2e0200e843ead0825d1b34d7446864e34"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:3e960b5185f637d7606eb01126d2543519138608"
status: active
---

# Hive 보존형 제거

`hive uninstall`: Hive host activation·projection·package·derived index·backup·transaction·runtime 제거.
`.hive/knowledge/`, 저장 user preference, foreign host entry, 비-Hive 사용자 파일 보존.
`--full`·`-f` mode 제공 없음. knowledge·preference 삭제: 사용자 수동 작업.

이후 `hive install --scope user --apply`: 보존된 preference 감지·Hive-owned user projection 복원·setup 질문
재표시 없는 user-scope setup 완료.
