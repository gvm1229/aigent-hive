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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2960ab32ce7831e993fee381e664d20eb5f673a62f006582a8e601476ccdb0fa"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:6606c09b03b9a0b3896a8b9242a937aec0a25a644ffbf873a3117e6c47410ccf"
  - "repo:docs/plans/active/user-onboarding-shared-index.md#sha256:889f7b0e5f374b1c78117486dcd24bd02df5d96b00c340402f2c672eb54b3b61"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:docs/state/CURRENT.md#sha256:71cdf8e460cbc56267611fc4967f489e1b9a535ca8b7b2aefe3b0fa77e8e6187"
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
- `.hive/dev-install`: 별도 developer rollback state
