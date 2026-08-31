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
  - "repo:.agents/directives/01-behavior.md#sha256:3a8450ff3e496f4e6bafc7b8d10cdd9fe38f15932b465d131a69ca0bdf9ef2f3"
  - "repo:.agents/directives/08-human-documentation-style.md#sha256:f1c700565caf1c448cfa0a7d58db549d5c3d466b264737233fe255c67663acd6"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:2a7c02ca89bc80f95574e9c6147af3d634bcfd3c40f395a554a225175bf09d91"
  - "repo:harness/project-bases/0.9.0/AGENTS.md.template#sha256:1aefece59d56d610227b64cfcfff8c634e47202f8e224916b248a8e8ecd9de51"
  - "repo:harness/skills/prompt-refine/SKILL.md#sha256:3848758e0725a7b9b990d3055f22942ec6aededee7d3c8255d0162c8633c6fc5"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:a6aea1ed5b977bc818bace5c9d712d2da01328f59753e9b93136c17b1a8f24d3"
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
