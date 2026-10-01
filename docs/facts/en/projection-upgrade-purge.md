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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:f2ff844d6567f78dde41ec78d1f030cd76a050cdd6fb977a4d8bf8401b2dffd7"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:6da6fa6180feb983e71a9be4915ff93a12cc1bfefae6016bad026f5c94b0238e"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:6f0d42adfc0aa7cf6e199898d950a7219e0bc3b3c78ef1bbc5b822d742e142dd"
  - "repo:crates/hive-update/src/merge.rs#sha256:a8eeefc6b27b42c7eb0c0795f4ca91b25401cbdfdd9f00064a629138a50e6283"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:dd6012ef058aa3f08690daed211ab690ef4bccbb8eb1fa3710eeee72a4dd9ffc"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:e6c5137a0c1e61dc0845202cbfa18421238ba0eef7550c08a3dd2f3bdd73cbc5"
links: [consumer-session-coordination, hive-preserving-uninstall]
reviewed_revision: "git:fc288bed8f925b89bfd0ed67b808cfdd0722a70b"
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
