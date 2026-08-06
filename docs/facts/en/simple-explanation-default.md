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
  - "repo:.agents/directives/01-behavior.md#sha256:b84d45bd3fb28b67581ff024fdfb8d0aaf0dfbbda580f712c96d1228685dadbb"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:35f3cace4b6297a298b8b59db208b3d8ecfd82331758fb6bd34dd1ec03aa8ec7"
  - "repo:docs/guidance-schema.md#sha256:fd8fffda818038ee48b66b0581787e2fd741404b0a9253ca34c0d55f15ad4d15"
  - "repo:harness/template/AGENTS.md.jinja#sha256:070d97440343d699565448c239efb55c905df79119df289525d41edc6e81581f"
links: [language-consistency, verification-result-clarity]
reviewed_revision: "git:35e6b79a024350487f823780101a28be24a9f4c7"
status: active
---

# Simple Explanation Default

Source-development and installed consumer agents explain in simple terms by
default. They add a concrete example when it materially improves understanding,
but do not force an irrelevant example or trade away technical precision.
Acceptance: source and consumer guidance producers, selected-language user
guidance, and direct regression checks carry the same bounded rule. Origin:
maintainer request to make this explanation style the default.
