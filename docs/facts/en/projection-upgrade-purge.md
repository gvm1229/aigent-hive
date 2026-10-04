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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:0827c9c337e692cd64767fca05256862950fbb1b765595a3d7431ff75d265380"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:42f69c326667ad73522caeadec761a523074a78efcb5caf05a74185acb0fe3ce"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ffaa44da03bc2179a4b9d7e743fb42d556506338072fe7de2d73dc3100409971"
  - "repo:crates/hive-update/src/merge.rs#sha256:a8eeefc6b27b42c7eb0c0795f4ca91b25401cbdfdd9f00064a629138a50e6283"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:8c252fa5ef5c4c40647cc11127a404f0bac7c096648e8b3ca9f8af9655203067"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:e6c5137a0c1e61dc0845202cbfa18421238ba0eef7550c08a3dd2f3bdd73cbc5"
links: [consumer-session-coordination, hive-preserving-uninstall]
reviewed_revision: "git:931cff4f9faa2e27093ba07b26f7e572657e4ca6"
status: active
---

# Authenticated Projection Upgrade Purge

Global setup removes retired Skills only when the retired-name ledger and shipped historical digest match. Project refresh uses the authenticated base: delete unmodified retired paths; preserve modified or foreign bytes.

Incoming safety or ownership rules replace overlapping Hive rules in directives and the exact AGENTS marker. Disjoint additions, foreign blocks and other local conflicts retain local priority. Preview, digest approval, atomic apply and rollback remain required.

The cleanup regression fix uses an actual validated removed file as the empty-directory ancestry witness, with no-follow handles. Real project validation and an unchanged second preview check the update; model compliance needs separate evidence.
