---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: ko
counterpart: ../en/global-onboarding.md
title: "Global onboarding"
summary: "전역 설정: 사용량 보호 활성화 권장, native 실패 전 CodexBar 비노출, 보존형 재설치."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:e42b36f0bfd2629b0abbbeb2010d8aae5bac3086982f1a714bb6888aef1a3364"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:0bfd9117a0d835da5f19bc02b82959a5630a4955a81ee3efda0a6ba5246dfaad"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
  - "repo:docs/decisions/product-release-decisions.md#sha256:7e6acd0973f56e3a15e4aad766a907c76a1511f6f5931c36f71ba8d979e90beb"
  - "repo:docs/plans/active/native-usage-sensor.md#sha256:0482703078d8876e4324383eea4865b1f0fbdea7495bd283c89147ed2e34d85d"
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:24f84ff3eeb32ba7d5ee5c449d34cf0bb80e300a6123742f22664797402ab219"
  - "repo:docs/plans/active/user-onboarding-shared-index.md#sha256:aafc3f1fb28a8e43939309d0dfb21586305759e2326d61fce444189f7acc79d1"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:6b2a26d1285073e6796f683abfc190bd6d74a05d57b83900412da37aa5d53849"
  - "repo:harness/user-setup/catalog.yml#sha256:4926655a12591cae061e674d774557e96f000d149f8dec1c2b1b650ba235f494"
  - "repo:schemas/user-setup.schema.json#sha256:e83e5f318a5b6ffcc08cfe0898a2b6138512c6bfb0eea99c6070b134f3712f47"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:01df1d580d987e7fb0f34978076cd000263fd99f"
status: active
---

# Global onboarding

- 사용량 보호: 활성화 권장. 신속 기본 남은 사용량 `20%`; custom 한도는 사용자 입력
- CodexBar: 정상 setup 비노출. Post-init native 실패 확정 뒤만 별도 동의
- Setup·update·uninstall: current Skill closure 수렴, retired empty shell 제거,
  knowledge·saved preference·foreign byte 보존
