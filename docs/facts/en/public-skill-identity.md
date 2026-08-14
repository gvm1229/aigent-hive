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
  - "repo:crates/hive-projection/src/lib.rs#sha256:99cb338be7955c854c0172ec984e917b797523c456e73f9d313d2594e8900b56"
  - "repo:docs/plans/active/knowledge-skill-naming-0.9.3.md#sha256:395a33fa2bbab8440265570dd1802605d2157ed0029b86fdc326a825ac1771d8"
  - "repo:docs/skills.md#sha256:d5b65f1bed7b9d4adeaf168df3dc349de9c20b4b0fb84e09a14be95084012a71"
  - "repo:harness/skills/catalog.yml#sha256:d23ab5c0d658f432c1f051352ce9f21b4646e85f3bd45df0105d5559f386481c"
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
