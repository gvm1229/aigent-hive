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
  - "repo:README.md#sha256:362f1c802d9f436ffc33682d07709ed9655ce8fa098085f8d930fba93a84888e"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:73d95136c28b6742d76d7aca0874144808500168a67fc97accdee9bc2b387481"
  - "repo:docs/archive/plans/foundations/native-usage-sensor.md#sha256:231e96967c32029d539eb82f245399e37156a43c2028be8a01a51215a5455807"
  - "repo:docs/archive/plans/foundations/usage-guard-policy.md#sha256:4b99d1f046ff56eeb9102b99dec4e88226ca2cdfa4947bb233c9a5c541a19172"
  - "repo:docs/archive/plans/foundations/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/archive/plans/foundations/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
  - "repo:docs/decisions/product-release-decisions.md#sha256:b83e0f586453e6aeb0356f5a4eee84591b2946ccb06ecbffde35131027878de6"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:1fcbb2b9b2db6d57bd40682f80db2a0a916ebbffb3434431038b609b6b743c11"
  - "repo:harness/user-setup/catalog.yml#sha256:3f24914859e7bcbe9bb8c85aafeee4250bdc2da383d0480d000a967fcb3305c5"
  - "repo:schemas/user-setup.schema.json#sha256:83427614c5b997a695b9f22c52093d4e2d26892b7eb42fc9873309891d0e81e0"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:8a99ccacb96a249692d01e6835efc872adb7fe95"
status: active
---

# Global Onboarding

- The usage guard is recommended; expedited setup uses 20% remaining, and custom setup asks.
- CodexBar stays hidden until a post-initialization native failure is confirmed.
- Setup, update, and uninstall converge to the current Skill closure, remove retired empty shells,
  and preserve knowledge, preferences, and foreign bytes.
