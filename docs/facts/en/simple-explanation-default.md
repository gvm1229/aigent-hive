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
  - "repo:.agents/directives/01-behavior.md#sha256:3a8450ff3e496f4e6bafc7b8d10cdd9fe38f15932b465d131a69ca0bdf9ef2f3"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:1518c1b9ac4f68d114a59603a490491221b0459e36137fb380d2c247f9e1ab1a"
  - "repo:docs/guidance-schema.md#sha256:eae385d284f448a27a5243d8e7846aa69d9568e0849d3457147fb814229416ad"
  - "repo:harness/template/AGENTS.md.jinja#sha256:7fdcd351b7d0624baa68d11bf9e850692c9eeaae13abb003295c8727f621543a"
links: [language-consistency, verification-result-clarity]
reviewed_revision: "git:15128a22d61452bb22fd8d9e9168acd9d26340f8"
status: active
---

# Simple Explanation Default

Start with user-visible files, settings or knowledge, and the safe next action.
Use `projection`, `manifest`, or `digest` only when diagnosis requires them,
with a plain definition in the same sentence. Lists use one Markdown entry per
line; independently selectable options never share a comma-separated paragraph.
