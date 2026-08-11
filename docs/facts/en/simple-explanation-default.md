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
  - "repo:.agents/directives/01-behavior.md#sha256:8ca3714e63a836e9099119cf2ead0913698c95e3473298afd00aab072d4ab0c6"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:d24cc8e55c8706144ac684cb7ccce3bfa9119c4bd0e20a3e6e36222d9d731eea"
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
