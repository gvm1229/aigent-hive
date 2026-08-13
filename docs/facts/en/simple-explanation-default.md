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
  - "repo:.agents/directives/01-behavior.md#sha256:9d8adb7c75015fd24df8cb226a16180548c600dc963ee154c0a4af408e9fa48c"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:db91b9841c057a3f9b964185fb2a2f3c2f8701908cf6439e26bf05d389a7243d"
  - "repo:docs/guidance-schema.md#sha256:eae385d284f448a27a5243d8e7846aa69d9568e0849d3457147fb814229416ad"
  - "repo:harness/template/AGENTS.md.jinja#sha256:9b45162b48b5d6eedcce0d0b6bc2d63613776fc1089a7d2878941897d0d506c5"
links: [language-consistency, verification-result-clarity]
reviewed_revision: "git:536f5076534cedcdb9ea3d118830792fe61cd75e"
status: active
---

# Simple Explanation Default

Start with user-visible files, settings or knowledge, and the safe next action.
Use `projection`, `manifest`, or `digest` only when diagnosis requires them,
with a plain definition in the same sentence. Lists use one Markdown entry per
line; independently selectable options never share a comma-separated paragraph.
