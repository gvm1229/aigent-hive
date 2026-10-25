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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:1ab23dbb5a8b60322f6c1068ba7cc09f54bc34151915068a748a1cc5a2c38018"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2fade132e2fe4e60b3f7b70e089d5deaa145a25e2976bdfdb62a58a82fa86192"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:9cf945c0072f953b590757cffec4fbda47b68ac0145cb4b573b220aa6ddc4d45"
  - "repo:crates/hive-update/src/merge.rs#sha256:d081f88fb284963e9ae02366ae3e99f76f9e65383948660ea59f403df89d741d"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:8c252fa5ef5c4c40647cc11127a404f0bac7c096648e8b3ca9f8af9655203067"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:b33ed5b0aa3819b3607100012f444faa6af82f7f1363c2ca492cb7eaf81bb1df"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:7a462e6037870907849f27be51f3f49b09c483612844ca3efadd95512ecb10ba"
links: [consumer-session-coordination, hive-preserving-uninstall]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Authenticated Projection Upgrade Purge

Global setup removes retired Skills only when the retired-name ledger and shipped historical digest match. Project refresh uses the authenticated base: delete unmodified retired paths; preserve modified or foreign bytes.

Incoming safety or ownership rules replace overlapping Hive rules in directives and the exact AGENTS marker. Disjoint additions, foreign blocks and other local conflicts retain local priority. Preview, digest approval, atomic apply and rollback remain required.

The cleanup regression fix uses an actual validated removed file as the empty-directory ancestry witness, with no-follow handles. Real project validation and an unchanged second preview check the update; model compliance needs separate evidence.
