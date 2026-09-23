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
  - "repo:.agents/directives/01-behavior.md#sha256:7679dd5603fdfa1104b0017e9fe7c7acb6a81f9e4095233ccddf3db01a325af8"
  - "repo:.agents/directives/08-human-documentation-style.md#sha256:cbf25afb23f5e326533902174eb649d1870b2912bda56731b414707f3359433a"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b57e2ac894503cf91d6efc41940284e79cc11e2ca70f6718d5b88926a917ec67"
  - "repo:docs/guidance-schema.md#sha256:eae385d284f448a27a5243d8e7846aa69d9568e0849d3457147fb814229416ad"
  - "repo:harness/template/AGENTS.md.jinja#sha256:02cfaea5d05fc9e51b5a368cc70290f6f2b041912433bf640ad2ccaa0f6b2c39"
links: [language-consistency, verification-result-clarity]
reviewed_revision: "git:4afd5ba4483d98f63ae42c065837f9b010506657"
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
