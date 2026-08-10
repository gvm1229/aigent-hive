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
  - "repo:README.md#sha256:a03aae178a8c1060d3f4301d4ed592a24e8cf9e9e95a7b87afa434804ad4ecbb"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ab8cfec03bc6fcfb7d0e55e5c47d5c5bc57fa75adcb1993cd55086f686b56741"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:d30564f33f2ead463cfe9e18aa68b697cb07b6c419ee42c9b583fcc11edaf966"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:bc5180991fddb1c2e4132fb0e6f23d4d4d06bd9d60311085d853481201e5c052"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:9ffa22cf14504ba7385135c1f62fdcb19bede32a0925ed72eb23fa8b96359eb5"
  - "repo:harness/user-setup/catalog.yml#sha256:4926655a12591cae061e674d774557e96f000d149f8dec1c2b1b650ba235f494"
  - "repo:schemas/user-setup.schema.json#sha256:e83e5f318a5b6ffcc08cfe0898a2b6138512c6bfb0eea99c6070b134f3712f47"
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
