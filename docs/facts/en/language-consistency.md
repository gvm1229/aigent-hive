---
schema_version: 1
pair_id: language-consistency
topic_slug: language-consistency
language: en
counterpart: ../ko/language-consistency.md
title: "Response Language Consistency"
summary: "English responses use ASD-STE100. Korean responses use Korean. Hive prompts default to English unless the user specifies the current prompt language."
tags: [communication, documentation, harness, language]
aliases: ["Korean response", "controlled English", "language consistency"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:7d8300e65cd3136b350aa96000437faff764cb11af33ed42dedf4c88579448ea"
  - "repo:.agents/directives/08-human-documentation-style.md#sha256:f1c700565caf1c448cfa0a7d58db549d5c3d466b264737233fe255c67663acd6"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:64e7ee1eb9aaafd399fe971ca35e5df6aee68285029a9b84fa6b928a3324ffdc"
  - "repo:harness/project-bases/0.9.0/AGENTS.md.template#sha256:1aefece59d56d610227b64cfcfff8c634e47202f8e224916b248a8e8ecd9de51"
  - "repo:harness/skills/prompt-refine/SKILL.md#sha256:3848758e0725a7b9b990d3055f22942ec6aededee7d3c8255d0162c8633c6fc5"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:1fcbb2b9b2db6d57bd40682f80db2a0a916ebbffb3434431038b609b6b743c11"
  - "repo:harness/template/AGENTS.md.jinja#sha256:7fdcd351b7d0624baa68d11bf9e850692c9eeaae13abb003295c8727f621543a"
links: [global-onboarding, source-development]
reviewed_revision: "git:64125db02505a9a696e870d23fa54feb125b8093"
status: active
---

# Response Language Consistency

Source agents and consumer harnesses use the selected response language. Only
an explicit request for the current response can change it.

A Hive-authored, refined, or copy-ready prompt uses English by default. An
explicit request for the current prompt language has priority. The surrounding
response keeps the selected response language.

English responses and default prompts use ASD-STE100 Simplified Technical
English. Korean responses use Korean vocabulary and sentence structure. Do not
use replaceable English, mixed Korean-English compounds, technical-sounding
English, or emphasis-only English parentheticals. Source and consumer contracts
contain the required examples and projection tests.
