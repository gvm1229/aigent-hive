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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:f601258ae8aefb9e1456dab1f983272b2074d02b0d862bfe26300afb13f1446b"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:cb42f6c3bd643bc236f3af89f4388ffdbc08db66af88123a38267b904d7b9d01"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:c28e40cf7e976d0b4455a73c2ddb6d598af448cd549569097784e9a26b2d678e"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:4f3676378fafac75f9c6376210c760a2e0200e843ead0825d1b34d7446864e34"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:4a30456eb0ff07336ec2996a0c562d0a7d7651d9"
status: active
---

# Hive 보존형 제거

`hive uninstall`: Hive host activation·projection·package·derived index·backup·transaction·runtime 제거.
`.hive/knowledge/`, 저장 user preference, foreign host entry, 비-Hive 사용자 파일 보존.
`--full`·`-f` mode 제공 없음. knowledge·preference 삭제: 사용자 수동 작업.

이후 `hive install --scope user --apply`: 보존된 preference 감지, setup 질문 재표시 없이 user-scope
setup 완료.
