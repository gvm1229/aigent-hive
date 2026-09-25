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
  - "repo:README.md#sha256:ba515cb5d9ee4f825305a99d0807b19bf608cb792eb768644776e3c0b4522735"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:e1e23470bdd37528700da00641cedef9e510c525863c991c9d189ab761e9bde1"
  - "repo:docs/archive/plans/foundations/native-usage-sensor.md#sha256:231e96967c32029d539eb82f245399e37156a43c2028be8a01a51215a5455807"
  - "repo:docs/archive/plans/foundations/usage-guard-policy.md#sha256:4b99d1f046ff56eeb9102b99dec4e88226ca2cdfa4947bb233c9a5c541a19172"
  - "repo:docs/archive/plans/foundations/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/archive/plans/foundations/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
  - "repo:docs/decisions/product-release-decisions.md#sha256:9f234ef3fede8030ab6ad57fa4b560a71f30f468622c4e4ad56b83864d0e05ce"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
  - "repo:harness/user-setup/catalog.yml#sha256:fed2aefc7efa52c28bb05c5b069ad4c4fbeec30b805fff7b84d00285fca18ea4"
  - "repo:schemas/user-setup.schema.json#sha256:9c4d51829f1c6bc8553ed566a9663907dd5746d08b94a3a76c7744e21b556548"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:4afd5ba4483d98f63ae42c065837f9b010506657"
status: active
---

# Global Onboarding

- The usage guard is recommended; expedited setup uses 20% remaining, and custom setup asks.
- CodexBar stays hidden until a post-initialization native failure is confirmed.
- Setup, update, and uninstall converge to the current Skill closure, remove retired empty shells,
  and preserve knowledge, preferences, and foreign bytes.
