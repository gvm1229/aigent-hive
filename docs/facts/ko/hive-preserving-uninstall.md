---
schema_version: 1
pair_id: hive-preserving-uninstall
topic_slug: hive-preserving-uninstall
language: ko
counterpart: ../en/hive-preserving-uninstall.md
title: "Hive 보존형 제거"
summary: "보존형 제거: Hive-owned transient·retired projection 완전 정리, knowledge·preference·foreign byte 보존."
tags: [bootstrap, onboarding, preservation, uninstall]
aliases: ["clean reinstall", "hive uninstall"]
sources:
  - "repo:crates/hive-cli/src/user_install.rs#sha256:4aecfd684f8c07326a639e92061de5f2ea52050cddc352a3b2f4b6b4adb1d3c2"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:5200ab01acbf0c0577e27de976b91c5a697dd83437a25ed94de3ec93c510dcf3"
  - "repo:docs/plans/active/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:docs/state/CURRENT.md#sha256:4fef77c8f30d16992b1330ecee2cd71b4efbdeea628721ab39aca629b03b83d3"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:90a8ecca713a1b1963b5f1863f76d32d5c5b9532ca72922c2705ee9b63520307"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:089b0717e24c368a1725774aaca0c85ab596df10"
status: active
---

# Hive 보존형 제거

- 제거: Hive activation·projection·package·index·backup·transaction·runtime
- 보존: knowledge·saved preference·foreign byte
- `test.19` Mac audit: retired empty `agents/` leaf 44개와 empty transaction directory 잔존
- 보정 완료: 설치·갱신·제거가 인증된 Hive-owned 빈 조상을 leaf-to-root로 정리
- Exact 44-leaf fixture와 `0.9.1` Windows preserving reinstall: retired empty `agents/` leaf·
  empty transaction directory `0건`
- `.hive/dev-install`: 별도 developer rollback state
