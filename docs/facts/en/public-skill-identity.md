---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: en
counterpart: ../ko/public-skill-identity.md
title: "Skill Identity"
summary: "Aigent Hive has a 27-Skill product-only catalog with stable English IDs, including humanize-kor for explicit Korean rewriting."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:crates/hive-projection/src/lib.rs#sha256:b79d42a472aedc3cc05ce9d5439aebd5c99171798cf9e26faca1c17ac0f3558a"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:76e70020fd1492cf59530fc27e1c537dca4c59ddb57bb241f9710e7b667cf535"
  - "repo:harness/skills/catalog.yml#sha256:76ed4b4d220db932da8e0e63aee700875f460b72922715a56b83bae1b9065273"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:eaed3203ce3fea062acab325a9ce0892348aff02"
status: active
---

# Skill Identity

Aigent Hive has one product-only catalog of 27 Skills. Existing English IDs remain stable for
execution and setup compatibility. `humanize-kor` adds explicit Korean rewriting with deterministic
preservation gates. Korean knowledge display names show only the human function, while every
description starts with its canonical ID. Historical release inventories remain immutable.
