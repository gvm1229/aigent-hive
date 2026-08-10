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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:4790c9666065f4bc49ebf0eaee4c50fce384a2fd44a69cd16670b9c9d6d7f39a"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:2dbd0f956fea6c6e258a275bc89565c48a7bf211819ea8816512215dc2582213"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:4f61520861d38b63448a45b91dd96443dfba20c79b3d8abade6099460956d3ed"
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
