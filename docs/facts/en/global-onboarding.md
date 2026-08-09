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
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:2150681617bd1c2273780f0796609f27fc4815418428c0743ef11b88245deb38"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:19537ff592e3146740b87256c5cd25033ceb8dbfc556c7da7f219baf1360666e"
  - "repo:harness/skills/configure/SKILL.md#sha256:abeb032e21d2576366025465d54080966767fb7e17cca57848acf093eaa83eaf"
  - "repo:harness/user-setup/catalog.yml#sha256:7dc82dbf559075ce4286e7dd19aec0ddc22e04f35ad4a8a60f43129a4dba2a1f"
  - "repo:schemas/user-setup.schema.json#sha256:46b360a9f91e154d1440e2997b56a964edd122383ccfc9b105b4e2ae4f8939f9"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
status: active
---

# Global Onboarding

Order: install CLI, activate host, run global setup, then set up a project. Global setup never
inspects a project. The user chooses the global usage threshold; a project may only stop earlier.

Windows 11 test.5 exposed CLI lookup failure, schema guessing, temporary answer files, and lost
progress. Repair requires verified CLI and signed setup metadata before questions, saved progress
after every answer, one cleaned OS-temp file, the product-only Skill catalog, and the shared
global/project guard. Stable 0.9.0 requires a fresh numbered-test pass on the maintainer's actual
Windows 11 machine. This Mac runs source and cross-platform regressions only. Unknown or edited
bytes stay unchanged.
