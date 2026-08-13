---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: en
counterpart: ../ko/public-skill-identity.md
title: "Skill Identity"
summary: "Aigent Hive has a 26-Skill product-only catalog; knowledge Skills show a human function label with their canonical English ID."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:crates/hive-projection/src/lib.rs#sha256:7b06c12e607a3e6ef8cf547fe8d6d2be67abf70edadfbb45b0612432a2a7b1ff"
  - "repo:docs/plans/active/knowledge-skill-naming-0.9.3.md#sha256:395a33fa2bbab8440265570dd1802605d2157ed0029b86fdc326a825ac1771d8"
  - "repo:docs/skills.md#sha256:89909ed6df13cf089302e226e4df2a27322dfcc0007292302434df18b7a85ae0"
  - "repo:harness/skills/catalog.yml#sha256:640f2ded6bb90de6c8c0797d21028091512569549478400b4721245c47ce3fae"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:da8ff786068c1cf28b0e40862494767ddeffe9c0"
status: active
---

# Skill Identity

Aigent Hive has one product-only catalog of 26 Skills. Existing English IDs remain stable for
execution and setup compatibility. Korean knowledge labels pair the human function with the ID,
and each description starts with `(knowledge-...)`. `knowledge-capture` keeps one safe useful
claim after a turn; recall searches current work, import scans a chosen repository, promote shares
reviewed knowledge, and maintain checks or explicitly cleans it.
