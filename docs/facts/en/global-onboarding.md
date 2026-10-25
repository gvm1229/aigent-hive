---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "New 0.14.0 setup disables automatic usage stops and reviews existing choices while preserving user data."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:049596fcbb0b062c936adef3eccc3fb154e407778a1c98bbfef6da20314fd07f"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:9cf945c0072f953b590757cffec4fbda47b68ac0145cb4b573b220aa6ddc4d45"
  - "repo:crates/hive-core/src/preferences.rs#sha256:838005e60b3255422c2ee143d76e66ef3ef877d7f3a8ff196eb446c0dfdc888d"
  - "repo:docs/archive/plans/foundations/native-usage-sensor.md#sha256:231e96967c32029d539eb82f245399e37156a43c2028be8a01a51215a5455807"
  - "repo:docs/archive/plans/foundations/usage-guard-policy.md#sha256:4b99d1f046ff56eeb9102b99dec4e88226ca2cdfa4947bb233c9a5c541a19172"
  - "repo:docs/archive/plans/foundations/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/archive/plans/foundations/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
  - "repo:docs/decisions/product-release-decisions.md#sha256:3620f38dc575abdd65508175d3a33b7e998b957d70b05c8c807337eafa5a0321"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:b33ed5b0aa3819b3607100012f444faa6af82f7f1363c2ca492cb7eaf81bb1df"
  - "repo:harness/user-setup/catalog.yml#sha256:fed2aefc7efa52c28bb05c5b069ad4c4fbeec30b805fff7b84d00285fca18ea4"
  - "repo:schemas/user-setup.schema.json#sha256:4f9c5f59802bc964cfdc41636c32e7e71929c52d64a0291babba948906567a1f"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Global Onboarding

- Before 0.14.0, quick setup recommended protection at 20% remaining; custom setup asked.
- New 0.14.0 setups disable automatic stops. Existing choices and unknown origins receive CLI review; unattended updates preserve choices.
- CodexBar stays hidden until a confirmed native failure and separate consent.
- Setup, update and uninstall keep the current Skill closure, prune authenticated retired empty shells, and preserve knowledge, preferences and foreign bytes.
