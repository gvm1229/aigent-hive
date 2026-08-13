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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:ceccb1021153bcc1a73c1b5f44ecb2ecee268a70db398412eaf75521ef83a337"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:db91b9841c057a3f9b964185fb2a2f3c2f8701908cf6439e26bf05d389a7243d"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:9fa9e439ad15ea6a8b5ed7cf6d031595a8979b056dada55360cb32331d9e8355"
  - "repo:crates/hive-update/src/merge.rs#sha256:4dc96d4c159d55be6664fa565dbb0eb77c1df532330f8a539f028ce51a9fcaaa"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:3810a0ce4919ccbcfd02961a1cefdd5f6329d938eed1b411d839edeac3b3a86b"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:de84c29ec9221b5b0bd531e31f21d5317424ed8bdb3f1cfe019670d7f2e876c4"
  - "repo:tests/conformance/test_phase3_static_contracts.py#sha256:a1480b59d9bdff0b7c5d27c50bf18dcb7a49f56835b872d4d5829580202a9db4"
links: [consumer-session-coordination, hive-preserving-uninstall]
reviewed_revision: "git:65f5a7df6d1abed4f9e299992d85e6377464b1d5"
status: active
---

# Authenticated Projection Upgrade Purge

Global setup removes a retired `.agents/skills/<name>/SKILL.md` only when the retired-name ledger
and a shipped historical Hive digest both match its active bytes. The project refresh path already
uses the authenticated project base inventory: an unmodified retired path absent from the incoming
projection is deleted, while modified or foreign bytes remain protected.

For Hive directives and the Hive-owned marker in `AGENTS.md`, an incoming rule with safety or
ownership content replaces an overlapping prior Hive rule. Disjoint user additions, foreign blocks,
and overlapping non-safety local rules retain local priority. Every refresh keeps preview, digest,
atomic apply, rollback, and empty owned-directory cleanup boundaries.
