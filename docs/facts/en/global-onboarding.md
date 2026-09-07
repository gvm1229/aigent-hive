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
  - "repo:README.md#sha256:41785eca2898e9c0a24770987e7533f1301bf23fdd7334b7b452a97189658fcd"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:633265271a82ee37ef07acd9e5a3d406ea80a3f17c7c9809d3d8d7619dd91260"
  - "repo:docs/archive/plans/foundations/native-usage-sensor.md#sha256:231e96967c32029d539eb82f245399e37156a43c2028be8a01a51215a5455807"
  - "repo:docs/archive/plans/foundations/usage-guard-policy.md#sha256:4b99d1f046ff56eeb9102b99dec4e88226ca2cdfa4947bb233c9a5c541a19172"
  - "repo:docs/archive/plans/foundations/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/archive/plans/foundations/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:d1105d7d3ebc2c5b8dbf100c4383ecd0933a8425c07501785b84ccdd4c6d2719"
  - "repo:harness/user-setup/catalog.yml#sha256:fed2aefc7efa52c28bb05c5b069ad4c4fbeec30b805fff7b84d00285fca18ea4"
  - "repo:schemas/user-setup.schema.json#sha256:daee52c6535601606bc39d67800ed2e6ad248828ac73383cc7d8ded015c95652"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:1b755a995d91739d758830210d93cdc012e9e61b"
status: active
---

# Global Onboarding

- The usage guard is recommended; expedited setup uses 20% remaining, and custom setup asks.
- CodexBar stays hidden until a post-initialization native failure is confirmed.
- Setup, update, and uninstall converge to the current Skill closure, remove retired empty shells,
  and preserve knowledge, preferences, and foreign bytes.
