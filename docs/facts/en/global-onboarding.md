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
  - "repo:README.md#sha256:e3b2bf7f1fa08e168811133d0e9992ff9e7477cb9a31cf24658a29a403a9b2dc"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:082645d4de44857f384e5613bbee10866928140c3932115fb3181baa7c3e27cb"
  - "repo:crates/hive-core/src/preferences.rs#sha256:cb912cc72fc576c28ddc2436597326957cdd58bca1810d5408e5127692825016"
  - "repo:docs/archive/plans/foundations/native-usage-sensor.md#sha256:231e96967c32029d539eb82f245399e37156a43c2028be8a01a51215a5455807"
  - "repo:docs/archive/plans/foundations/usage-guard-policy.md#sha256:4b99d1f046ff56eeb9102b99dec4e88226ca2cdfa4947bb233c9a5c541a19172"
  - "repo:docs/archive/plans/foundations/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/archive/plans/foundations/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
  - "repo:docs/decisions/product-release-decisions.md#sha256:3620f38dc575abdd65508175d3a33b7e998b957d70b05c8c807337eafa5a0321"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:005f05fa4631834f57fd4e8524e51a839d030e88ad1cf193c831a176fce03848"
  - "repo:harness/user-setup/catalog.yml#sha256:4c3a188bcdb06752531530478c34cedafcaafb259fff00ff8743ed2d2e4c85c0"
  - "repo:schemas/user-setup.schema.json#sha256:02903b420365725d94b3cb79000661970338767fe382f163a36c9d58e5288aa8"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Global Onboarding

- Before 0.14.0, quick setup recommended protection at 20% remaining; custom setup asked.
- New 0.14.0 setups disable automatic stops. Existing choices and unknown origins receive CLI review; unattended updates preserve choices.
- CodexBar stays hidden until a confirmed native failure and separate consent.
- Setup, update and uninstall keep the current Skill closure, prune authenticated retired empty shells, and preserve knowledge, preferences and foreign bytes.

- Supplied choices fill only answers in the CLI-provided `answer_patch_template`, preserving its kind and schema version; pass the saved file with `--answers`. Never infer choices or invent field keys from labels.
