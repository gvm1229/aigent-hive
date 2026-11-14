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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:55c9618f6cc2f1e5b18c00c2fc0c8260c6ada6c2020638a0ac2e81d6c0e4e4e0"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:9903f35f13ae28c9853cdf99ab17c42876e8a5b12c91100e7f4c51cc04147c09"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:34e8f7c0d827e90ed98a1b8832f8a2ab4b19f736ea67135f6fe71f728bf87287"
  - "repo:crates/hive-update/src/merge.rs#sha256:f9c983dc6fb71aa50994332395faaa55058d8e95add818221bed9c8073510558"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:8c252fa5ef5c4c40647cc11127a404f0bac7c096648e8b3ca9f8af9655203067"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:828eb911da5e50ef6cadd88c301e07f5aa454c2bbe1c9bcd7ef460f7efae00fe"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:edf491a6826fe3c171e84b8ec3b25bc41d5be32a397840fd91cbdbd74a947985"
links: [consumer-session-coordination, hive-preserving-uninstall]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Authenticated Projection Upgrade Purge

Global setup removes retired Skills only when the retired-name ledger and shipped historical digest match. Project refresh uses the authenticated base: delete unmodified retired paths; preserve modified or foreign bytes.

Incoming safety or ownership rules replace overlapping Hive rules in directives and the exact AGENTS marker. Disjoint additions, foreign blocks and other local conflicts retain local priority. Preview, digest approval, atomic apply and rollback remain required.

The cleanup regression fix uses an actual validated removed file as the empty-directory ancestry witness, with no-follow handles. Real project validation and an unchanged second preview check the update; model compliance needs separate evidence.
