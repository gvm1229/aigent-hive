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
  - "repo:.agents/directives/01-behavior.md#sha256:4b22be47789033b39654596bb345fd56017e54bf4cd8ef12ad1cac7ae9c8e4d4"
  - "repo:.agents/directives/08-human-documentation-style.md#sha256:a4f9c9d280a596786fb93cd0ee71bc7b5987f3bafe3be99b1184997f7af6465f"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:7a5c873834ba9a77e6efdedc60a5eed953fa40102dfcf88c084db5b591f465c3"
  - "repo:docs/guidance-schema.md#sha256:eae385d284f448a27a5243d8e7846aa69d9568e0849d3457147fb814229416ad"
  - "repo:harness/template/AGENTS.md.jinja#sha256:7fdcd351b7d0624baa68d11bf9e850692c9eeaae13abb003295c8727f621543a"
links: [language-consistency, verification-result-clarity]
reviewed_revision: "git:0f4f3ace47227fe88569340e763e3fcea9bc3f05"
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
