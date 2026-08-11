---
schema_version: 1
pair_id: hive-preserving-uninstall
topic_slug: hive-preserving-uninstall
language: ko
counterpart: ../en/hive-preserving-uninstall.md
title: "Hive 보존형 제거"
summary: "구조상 유효하지만 인증된 release와 불일치한 user-scope ownership manifest: 별도 승인 없이 보존형 재설치, knowledge base·저장 user preference 항상 보존."
tags: [bootstrap, onboarding, preservation, uninstall]
aliases: ["clean reinstall", "hive uninstall"]
sources:
  - "repo:crates/hive-cli/src/user_install.rs#sha256:d24cc8e55c8706144ac684cb7ccce3bfa9119c4bd0e20a3e6e36222d9d731eea"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:0bfd9117a0d835da5f19bc02b82959a5630a4955a81ee3efda0a6ba5246dfaad"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:6b2a26d1285073e6796f683abfc190bd6d74a05d57b83900412da37aa5d53849"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:089b0717e24c368a1725774aaca0c85ab596df10"
status: active
---

# Hive 보존형 제거

`hive uninstall`: Hive activation·projection·package·index·backup·transaction·runtime 제거.
`.hive/knowledge/`, 저장 preference, foreign entry, 비-Hive 파일 보존. `--full`·`-f` 없음.
knowledge·preference 삭제: 사용자 수동 작업.

`hive install --scope user --apply`: 저장 preference 기반 projection 복원, setup 질문 재표시 없음.
인증된 release와 불일치한 구조상 유효 manifest: 이미 승인된 install·update·setup의 preserving
reinstall 자동 실행, 추가 승인 없음. malformed·path-unsafe manifest, foreign overwrite, material choice:
사용자 결정.
