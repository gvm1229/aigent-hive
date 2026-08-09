---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "Global setup must verify and resolve the signed CLI before questions, preserve every answer, and own the user-selected usage-guard threshold while projects may request only earlier stops."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:67c09e54e76df72ee9ac6acbde5b88fbb0a6653e1d7172e3f789a8d99c2434b7"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:48a76cd5503858a2327c7562879de259334b182687aace98ec1df06b71dd1600"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:d30564f33f2ead463cfe9e18aa68b697cb07b6c419ee42c9b583fcc11edaf966"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:59812ce78d64825be25dbb6576013869e4d334b82868f663281c42c0b1df4e16"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:246f1fa6c352c29a905d4e3981312a2288e701785fb9d95c450d4023a37a059b"
  - "repo:harness/user-setup/catalog.yml#sha256:7dc82dbf559075ce4286e7dd19aec0ddc22e04f35ad4a8a60f43129a4dba2a1f"
  - "repo:schemas/user-setup.schema.json#sha256:46b360a9f91e154d1440e2997b56a964edd122383ccfc9b105b4e2ae4f8939f9"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:35f5bce71814a3e874fe53a8730024f16013ad46"
status: active
---

# Global Onboarding

Order: install CLI, activate host, run global setup, then project setup. Global setup never
inspects projects. Users choose the global usage threshold; projects may only stop earlier.

Mac developer/public-build original recovery is complete under `BGR-008–013`; current validation
passes. The combined plan keeps it as a regression gate, not new implementation.

Windows test.5 exposed CLI lookup failure, schema guesses, temporary files, and lost progress.
Repair requires verified CLI metadata before questions, saved progress, one cleaned OS-temp file,
the product-only Skill catalog, and the shared usage guard. Stable 0.9.0 requires a fresh test on
the maintainer's Windows 11 machine. This Mac runs source regressions only. Unknown or edited bytes
stay unchanged.
