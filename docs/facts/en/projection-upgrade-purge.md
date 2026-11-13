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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:945222ab2da1310babab674c81b28304d03015271720795afde44abd2324e0e1"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:193eb021ec2fc367176127ae05ab9dc2baea6097819ddd63de43d9a79ee7cc9a"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:b7a409e044561428f1bda83e8daef5a647a63c23a8f1d1426baffb336b63323a"
  - "repo:crates/hive-update/src/merge.rs#sha256:f9c983dc6fb71aa50994332395faaa55058d8e95add818221bed9c8073510558"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:8c252fa5ef5c4c40647cc11127a404f0bac7c096648e8b3ca9f8af9655203067"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:828eb911da5e50ef6cadd88c301e07f5aa454c2bbe1c9bcd7ef460f7efae00fe"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:746a446b6d86020d9536cdbea8bff82d822075338d8f4bbfdaa3202f5dafffd0"
links: [consumer-session-coordination, hive-preserving-uninstall]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Authenticated Projection Upgrade Purge

Global setup removes retired Skills only when the retired-name ledger and shipped historical digest match. Project refresh uses the authenticated base: delete unmodified retired paths; preserve modified or foreign bytes.

Incoming safety or ownership rules replace overlapping Hive rules in directives and the exact AGENTS marker. Disjoint additions, foreign blocks and other local conflicts retain local priority. Preview, digest approval, atomic apply and rollback remain required.

The cleanup regression fix uses an actual validated removed file as the empty-directory ancestry witness, with no-follow handles. Real project validation and an unchanged second preview check the update; model compliance needs separate evidence.
