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
  - "repo:.agents/directives/01-behavior.md#sha256:e92fc32054100c81742fa37aac4354bd971feafe62d887b7b6c8f6aa65882e49"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:037c9a991a929a76aee9da633a6ecb666e3b45d02315c0e721934eccedd2245a"
  - "repo:docs/guidance-schema.md#sha256:fd8fffda818038ee48b66b0581787e2fd741404b0a9253ca34c0d55f15ad4d15"
  - "repo:harness/template/AGENTS.md.jinja#sha256:bb858b1021be8b3fd9fc282820a34a4e923dea6a47e01bdddcf9745510c1381d"
links: [language-consistency, verification-result-clarity]
reviewed_revision: "git:35e6b79a024350487f823780101a28be24a9f4c7"
status: active
---

# Simple Explanation Default

Source-development and installed consumer agents explain in simple terms by
default. They add a concrete example when it materially improves understanding,
but do not force an irrelevant example or trade away technical precision.
Every user-facing list uses one Markdown entry per line; independently selectable
options never share a comma-separated paragraph.
Acceptance: source and consumer guidance producers, selected-language user
guidance, and direct regression checks carry the same bounded rule. Origin:
maintainer request to make this explanation style the default.
