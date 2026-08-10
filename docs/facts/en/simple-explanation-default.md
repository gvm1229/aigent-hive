---
schema_version: 1
pair_id: simple-explanation-default
topic_slug: simple-explanation-default
language: en
counterpart: ../ko/simple-explanation-default.md
title: "Simple Explanation Default"
summary: "Source agents start with the user-visible files, data effect, and safe next action before defining internal implementation terms."
tags: [communication, guidance, projection]
aliases: ["concrete examples", "plain-language explanation"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:37e56019efc3b863734395bde48de0c2e1a3abf4aab8b2be982533fcb2ef6097"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:4954a37de473c03e9b95f45c5b494cf40f7f01f2b23ba21f5b3d3bd3014650f2"
  - "repo:docs/guidance-schema.md#sha256:f5fc6aa2c36274d78d9703693a362c2f8d8eb81204d37f8a224434c14d1b196b"
  - "repo:harness/template/AGENTS.md.jinja#sha256:d706dc6585c1bbaa820d328ebfaae919cd02496adac0acec373ee4d0e37afe56"
links: [language-consistency, verification-result-clarity]
reviewed_revision: "git:536f5076534cedcdb9ea3d118830792fe61cd75e"
status: active
---

# Simple Explanation Default

Start with user-visible files, settings or knowledge, and the safe next action.
Use `projection`, `manifest`, or `digest` only when diagnosis requires them,
with a plain definition in the same sentence. Lists use one Markdown entry per
line; independently selectable options never share a comma-separated paragraph.
