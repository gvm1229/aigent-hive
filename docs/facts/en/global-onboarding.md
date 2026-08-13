---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "Global setup recommends the usage guard, hides CodexBar until native failure, and preserves user data across reinstall."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:5d1d0eecdd325aac99915b8838400668cdf2f34aff63e33b2b5f79923c877ebc"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:cffd6c491ffd17dccefa84edb172bbfe64ae925f2fe9cf7c6efd07e6a896a9fd"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
  - "repo:docs/decisions/product-release-decisions.md#sha256:7e6acd0973f56e3a15e4aad766a907c76a1511f6f5931c36f71ba8d979e90beb"
  - "repo:docs/plans/active/native-usage-sensor.md#sha256:8131d6eba753cae4bfc38ec30013a44385c92b50ed29a57de8a96c8b7395c246"
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:4b99d1f046ff56eeb9102b99dec4e88226ca2cdfa4947bb233c9a5c541a19172"
  - "repo:docs/plans/active/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:aa0e9102c6d4a08a2468f39abf66f2788844c28a989eace52f59f9d2ea919957"
  - "repo:harness/user-setup/catalog.yml#sha256:3f24914859e7bcbe9bb8c85aafeee4250bdc2da383d0480d000a967fcb3305c5"
  - "repo:schemas/user-setup.schema.json#sha256:83427614c5b997a695b9f22c52093d4e2d26892b7eb42fc9873309891d0e81e0"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:01df1d580d987e7fb0f34978076cd000263fd99f"
status: active
---

# Global Onboarding

- The usage guard is recommended; expedited setup uses 20% remaining, and custom setup asks.
- CodexBar stays hidden until a post-initialization native failure is confirmed.
- Setup, update, and uninstall converge to the current Skill closure, remove retired empty shells,
  and preserve knowledge, preferences, and foreign bytes.
