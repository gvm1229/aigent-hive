---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: en
counterpart: ../ko/public-skill-identity.md
title: "Skill Identity"
summary: "Aigent Hive has 28 product Skills, direct rename aliases, and separate knowledge scanning and transfer roles."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:crates/hive-projection/src/lib.rs#sha256:1e1231e9fd9dec40cb72167f734788f1521177027bd2d2bde44ca94d9ec2be74"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:b1d168024659e23bc1fee30c46e2b628e607522b9b0da2f59229a277eff2a702"
  - "repo:harness/skills/catalog.yml#sha256:5949525e029f37e08f5ef49302f698be45674b94959f4f5aa301d7138c4e1570"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:eaed3203ce3fea062acab325a9ce0892348aff02"
status: active
---

# Skill Identity

Aigent Hive has one product-only catalog of 28 Skills. Renamed IDs retain direct migration aliases;
`knowledge-scan` extracts new knowledge and `knowledge-transfer` moves existing knowledge.
`humanize-kor` adds explicit Korean rewriting with deterministic
preservation gates. Korean knowledge display names show only the human function, while every
description starts with its canonical ID. Historical release inventories remain immutable.
