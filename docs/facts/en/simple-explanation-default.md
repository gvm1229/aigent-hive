---
schema_version: 1
pair_id: simple-explanation-default
topic_slug: simple-explanation-default
language: en
counterpart: ../ko/simple-explanation-default.md
title: "Simple Explanation Default"
summary: "Source agents and installed user guidance explain replies and explanatory writing at a five-year-old comprehension level while preserving technical names, accuracy, and limits."
tags: [communication, guidance, projection]
aliases: ["concrete examples", "plain-language explanation"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:49c79d8137a1e1d3cdcb06b60fb7e2708e2cf0ae04018a6c9c866ecb0a65a333"
  - "repo:.agents/directives/08-human-documentation-style.md#sha256:8e045ba8ad7674019e53f7df9ac42b0b8a039af9a5907efc257ea85186e54e0f"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:07c53684b15ca76c0912c67ab9d2dac2464a98580fa186956492a95fa238af63"
  - "repo:docs/guidance-schema.md#sha256:eae385d284f448a27a5243d8e7846aa69d9568e0849d3457147fb814229416ad"
  - "repo:harness/template/AGENTS.md.jinja#sha256:02cfaea5d05fc9e51b5a368cc70290f6f2b041912433bf640ad2ccaa0f6b2c39"
links: [language-consistency, verification-result-clarity]
reviewed_revision: "git:e22ff9036b04535dc9a9cc542f08b20143cfe40e"
status: active
---

# Simple Explanation Default

Start with user-visible files, settings or knowledge, and the safe next action.
Source agents and installed user guidance apply this to replies, guides, blogs, reports, and other explanations:
familiar words, short sentences, one idea at a time, purpose then how and why. Define core terms
such as `digest` on first use. Use helpful examples, analogies, steps, or comparisons without baby
talk or forced length. Preserve numbers, commands, conditions, uncertainty, and evidence limits.
Check comprehension before sending or saving. Source `01-behavior` owns this policy; `08` refers
to it. Lists keep one entry per line and separate independently selectable options.
