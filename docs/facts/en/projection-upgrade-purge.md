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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:af09aadf2ddfabc082dfac9ae6c8233c2fe48f964db8996063848838f04f68c5"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:359e033f6bad6a6145820efb0a079a6643d4774a6d9b8e1b560d9d4e156df5be"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:64e7ee1eb9aaafd399fe971ca35e5df6aee68285029a9b84fa6b928a3324ffdc"
  - "repo:crates/hive-update/src/merge.rs#sha256:4dc96d4c159d55be6664fa565dbb0eb77c1df532330f8a539f028ce51a9fcaaa"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:acb330569b20bdfe3aa993ade2a07e0142e1fe5f981074b5bb506f647e8e97c6"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:1fcbb2b9b2db6d57bd40682f80db2a0a916ebbffb3434431038b609b6b743c11"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:be88ce1d1993eefdaafe3d2499d855f8a41a73a29cfcc7dfba22864a9e8739a0"
links: [consumer-session-coordination, hive-preserving-uninstall]
reviewed_revision: "git:47d4663f1e1f263276f9ce54b7c69a3ff95d2170"
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
