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
  - "repo:.agents/directives/01-behavior.md#sha256:24e61b7fd37bc1b9e0a73933547d5b369b9ca2cdde6c9adc10ba29bd23d50143"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:39bc19a47799793c2f2e984f5d7d6edb4e18fbbd96ec33ac30e7c258fda66d0b"
  - "repo:docs/guidance-schema.md#sha256:fd8fffda818038ee48b66b0581787e2fd741404b0a9253ca34c0d55f15ad4d15"
  - "repo:harness/template/AGENTS.md.jinja#sha256:070d97440343d699565448c239efb55c905df79119df289525d41edc6e81581f"
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
