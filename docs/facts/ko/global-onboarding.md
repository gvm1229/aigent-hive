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
  - "repo:README.md#sha256:3c390ad3b1a884c49a15304b0a0799299384e2e319e626ff7a752ecf4d700d94"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:5200ab01acbf0c0577e27de976b91c5a697dd83437a25ed94de3ec93c510dcf3"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
  - "repo:docs/decisions/product-release-decisions.md#sha256:7e6acd0973f56e3a15e4aad766a907c76a1511f6f5931c36f71ba8d979e90beb"
  - "repo:docs/plans/active/native-usage-sensor.md#sha256:8131d6eba753cae4bfc38ec30013a44385c92b50ed29a57de8a96c8b7395c246"
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:4b99d1f046ff56eeb9102b99dec4e88226ca2cdfa4947bb233c9a5c541a19172"
  - "repo:docs/plans/active/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:90a8ecca713a1b1963b5f1863f76d32d5c5b9532ca72922c2705ee9b63520307"
  - "repo:harness/user-setup/catalog.yml#sha256:4926655a12591cae061e674d774557e96f000d149f8dec1c2b1b650ba235f494"
  - "repo:schemas/user-setup.schema.json#sha256:57a426a58c822271f1c6297c2c607e532e83c5652ca92ef68bdbcd8b95d357fd"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:01df1d580d987e7fb0f34978076cd000263fd99f"
status: active
---

# Global onboarding

- 사용량 보호: 활성화 권장. 신속 기본 남은 사용량 `20%`; custom 한도는 사용자 입력
- CodexBar: 정상 setup 비노출. Post-init native 실패 확정 뒤만 별도 동의
- Setup·update·uninstall: current Skill closure 수렴, retired empty shell 제거,
  knowledge·saved preference·foreign byte 보존
