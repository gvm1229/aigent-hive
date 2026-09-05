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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:a9535c1ae9e207b08dfce0d71fe9293e38168d8c7a38a4187ce8c6752be90ce4"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:5a7d279c9ce96bec6792f191e4be0caa69bf17c9062ad535e11478af7f4408c2"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:b7226be968eac3e205183c1f0c7967066ba0b79f630f5d9715517824ea548b02"
  - "repo:crates/hive-update/src/merge.rs#sha256:4dc96d4c159d55be6664fa565dbb0eb77c1df532330f8a539f028ce51a9fcaaa"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:acb330569b20bdfe3aa993ade2a07e0142e1fe5f981074b5bb506f647e8e97c6"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:914cca3de8883e2b1be0dfbea92da3dd2c856cdca53ed24d3bd45d9ff75b6cd2"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:6158ac21248a2751c232af381793a50ca85806b2652a3a01adb5c3961e0327c5"
links: [consumer-session-coordination, hive-preserving-uninstall]
reviewed_revision: "git:15128a22d61452bb22fd8d9e9168acd9d26340f8"
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
