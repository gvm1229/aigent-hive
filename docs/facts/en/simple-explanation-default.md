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
  - "repo:.agents/directives/01-behavior.md#sha256:54f993c4403e7de3612efa12f499bbf1c3a957ee5893875cb0c50204505b1f40"
  - "repo:.agents/directives/08-human-documentation-style.md#sha256:8e045ba8ad7674019e53f7df9ac42b0b8a039af9a5907efc257ea85186e54e0f"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:193eb021ec2fc367176127ae05ab9dc2baea6097819ddd63de43d9a79ee7cc9a"
  - "repo:docs/guidance-schema.md#sha256:eae385d284f448a27a5243d8e7846aa69d9568e0849d3457147fb814229416ad"
  - "repo:harness/template/AGENTS.md.jinja#sha256:fe7f06cc88b566a9ab1ef2919d8ca43830aa963a98208c084c56b5c12f26c864"
links: [language-consistency, verification-result-clarity]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
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
