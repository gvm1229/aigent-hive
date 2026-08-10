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
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:0e688585dd2322a10687edcf5902ee99a9871728c251a52ca574f8aaf8105934"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:0d400b981b7680659c7588c777b23ef5a850b71681d46eadcf5fd49c08c0e793"
  - "repo:harness/user-setup/catalog.yml#sha256:4926655a12591cae061e674d774557e96f000d149f8dec1c2b1b650ba235f494"
  - "repo:schemas/user-setup.schema.json#sha256:e83e5f318a5b6ffcc08cfe0898a2b6138512c6bfb0eea99c6070b134f3712f47"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:35f5bce71814a3e874fe53a8730024f16013ad46"
status: active
---

# Global Onboarding

Order: install CLI, activate host, global setup, then project setup. Global setup never inspects
projects; users choose its usage threshold and projects may only stop earlier. Windows recovery
requires verified CLI metadata, saved progress, one cleaned OS-temp file, product-only Skills, and
the shared guard. An authenticated incomplete Hive marketplace activation is repaired silently
before setup resumes; canonical knowledge, saved preferences, and foreign host entries stay intact.
Stable 0.9.0 still requires a fresh Windows 11 test; source regressions do not replace it.
