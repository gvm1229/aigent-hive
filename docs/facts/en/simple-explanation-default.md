---
schema_version: 1
pair_id: simple-explanation-default
topic_slug: simple-explanation-default
language: en
counterpart: ../ko/simple-explanation-default.md
title: "Simple Explanation Default"
summary: "Source and consumer agents explain in simple terms by default and use concrete examples only when they help without reducing technical precision."
tags: [communication, guidance, projection]
aliases: ["concrete examples", "plain-language explanation"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:d1e3d4cbc89c962bfae66b5a9c135562bd962fa0d8a3765ad2d150e4a9e41195"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:f358b71778a5da093ffdad11470a7f4367573f037f219e4a497dc877bbd86f35"
  - "repo:docs/guidance-schema.md#sha256:fd8fffda818038ee48b66b0581787e2fd741404b0a9253ca34c0d55f15ad4d15"
  - "repo:harness/template/AGENTS.md.jinja#sha256:c53d41177ef323c50041c8e02928fd1db9904188c22d652d5a80dfbd454228e5"
links: [language-consistency, verification-result-clarity]
reviewed_revision: "git:33f365d3dbb1af51333a6dbb1834ce437a932ea0"
status: active
---

# Simple Explanation Default

Source-development and installed consumer agents explain in simple terms by
default. They add a concrete example when it materially improves understanding,
but do not force an irrelevant example or trade away technical precision.
Acceptance: source and consumer guidance producers, selected-language user
guidance, and direct regression checks carry the same bounded rule. Origin:
maintainer request to make this explanation style the default.
