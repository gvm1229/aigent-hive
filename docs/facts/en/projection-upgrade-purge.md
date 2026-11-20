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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:81c8223df0cba01511f0da69e8bc2b79bca26a22fa0439af874f6b4ed320606e"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:961f63ac2939655e614a95c1c0a950d86cc7cbb753402e78bf8e53674b0f9dca"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:082645d4de44857f384e5613bbee10866928140c3932115fb3181baa7c3e27cb"
  - "repo:crates/hive-update/src/merge.rs#sha256:f9c983dc6fb71aa50994332395faaa55058d8e95add818221bed9c8073510558"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:8c252fa5ef5c4c40647cc11127a404f0bac7c096648e8b3ca9f8af9655203067"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:005f05fa4631834f57fd4e8524e51a839d030e88ad1cf193c831a176fce03848"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:edf491a6826fe3c171e84b8ec3b25bc41d5be32a397840fd91cbdbd74a947985"
links: [consumer-session-coordination, hive-preserving-uninstall]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Authenticated Projection Upgrade Purge

Global setup removes retired Skills only when the retired-name ledger and shipped historical digest match. Project refresh uses the authenticated base: delete unmodified retired paths; preserve modified or foreign bytes.

Incoming safety or ownership rules replace overlapping Hive rules in directives and the exact AGENTS marker. Disjoint additions, foreign blocks and other local conflicts retain local priority. Preview, digest approval, atomic apply and rollback remain required.

The cleanup regression fix uses an actual validated removed file as the empty-directory ancestry witness, with no-follow handles. Real project validation and an unchanged second preview check the update; model compliance needs separate evidence.
