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
  - "repo:.agents/directives/01-behavior.md#sha256:54f993c4403e7de3612efa12f499bbf1c3a957ee5893875cb0c50204505b1f40"
  - "repo:.agents/directives/08-human-documentation-style.md#sha256:8e045ba8ad7674019e53f7df9ac42b0b8a039af9a5907efc257ea85186e54e0f"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:3c6e0b3a0f62d5598e68024fd7a69d82cb2b0dcfbb29d5602f6f362bc0df2d40"
  - "repo:harness/project-bases/0.9.0/AGENTS.md.template#sha256:1aefece59d56d610227b64cfcfff8c634e47202f8e224916b248a8e8ecd9de51"
  - "repo:harness/skills/prompt-refine/SKILL.md#sha256:bbd9a76fed57e1276aa94266709d79f656eafebc98e58723c5f8286b979399ed"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:7c483d40d8ad13ba52cdc9a7d1c10928320853530cd1a5abe5d3df8b8dd6cee7"
  - "repo:harness/template/AGENTS.md.jinja#sha256:ab0902d90e75ac32219217b3617c46008f3a906a84a68453bf389fd69e9ec529"
links: [global-onboarding, source-development]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
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
