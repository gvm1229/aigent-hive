---
schema_version: 1
pair_id: projection-upgrade-purge
topic_slug: projection-upgrade-purge
language: en
counterpart: ../ko/projection-upgrade-purge.md
title: "Authenticated Projection Upgrade Purge"
summary: "Hive removes retired Skills and replaces direct safety or ownership conflicts only after authenticating the prior Hive projection."
tags: [consumer-harness, preservation, skills, upgrade]
aliases: ["PUG93"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:30e453d8375c22b0ec9ec56998fc472dace3fff8cea63ad7ad360d5ff4c19898"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:7b8afc5527b300e665054ccd0f1fc6f34df02fee75e5e87f7198100cfd55caa9"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ffaa44da03bc2179a4b9d7e743fb42d556506338072fe7de2d73dc3100409971"
  - "repo:crates/hive-update/src/merge.rs#sha256:a8eeefc6b27b42c7eb0c0795f4ca91b25401cbdfdd9f00064a629138a50e6283"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:8c252fa5ef5c4c40647cc11127a404f0bac7c096648e8b3ca9f8af9655203067"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:461be3769de1d76f7dee1aefc4f2cae3b360a339f40b41ef13cff404f571b2dd"
links: [consumer-session-coordination, hive-preserving-uninstall]
reviewed_revision: "git:e22ff9036b04535dc9a9cc542f08b20143cfe40e"
status: active
---

# Authenticated Projection Upgrade Purge

Global setup removes retired Skills only when the retired-name ledger and shipped historical digest match. Project refresh uses the authenticated base: delete unmodified retired paths; preserve modified or foreign bytes.

Incoming safety or ownership rules replace overlapping Hive rules in directives and the exact AGENTS marker. Disjoint additions, foreign blocks and other local conflicts retain local priority. Preview, digest approval, atomic apply and rollback remain required.

The cleanup regression fix uses an actual validated removed file as the empty-directory ancestry witness, with no-follow handles. Real project validation and an unchanged second preview check the update; model compliance needs separate evidence.
