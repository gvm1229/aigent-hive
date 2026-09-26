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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:08d4aa0959ccc377a3f96a4c6f37df6f71c1473a271b406f7eb3b214f860cf0c"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b57e2ac894503cf91d6efc41940284e79cc11e2ca70f6718d5b88926a917ec67"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:e1e23470bdd37528700da00641cedef9e510c525863c991c9d189ab761e9bde1"
  - "repo:crates/hive-update/src/merge.rs#sha256:a8eeefc6b27b42c7eb0c0795f4ca91b25401cbdfdd9f00064a629138a50e6283"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:31cf73d71c42e548978b4e80dcc24820546e40f8e1d39cabb5eaaa4eaa78a1ff"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:37292dc27a67c3bb57204a752ed41ad04fe2ffb2ea6d014d7728908e87d588fa"
links: [consumer-session-coordination, hive-preserving-uninstall]
reviewed_revision: "git:4afd5ba4483d98f63ae42c065837f9b010506657"
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
