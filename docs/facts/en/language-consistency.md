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
  - "repo:.agents/directives/01-behavior.md#sha256:42bbd59e702cdce48ac6396d4c5a2f3a9b7574cd99272e22f3279c00b041cba4"
  - "repo:.agents/directives/08-human-documentation-style.md#sha256:f1c700565caf1c448cfa0a7d58db549d5c3d466b264737233fe255c67663acd6"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:b1372a8f956b74320081581b95db7333782e81bec926c2383a6fdc6f1f1dd884"
  - "repo:harness/project-bases/0.9.0/AGENTS.md.template#sha256:1aefece59d56d610227b64cfcfff8c634e47202f8e224916b248a8e8ecd9de51"
  - "repo:harness/skills/prompt-refine/SKILL.md#sha256:13e83b4b2a5a4605f53fb5f12af60dabe961fddf680771024ce300f762541486"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:f5d9b13356fb64171213e98b41045955760247c6f5e1ce420c991afe450063de"
  - "repo:harness/template/AGENTS.md.jinja#sha256:33c0da7ba5156ea1aa0ccc08a8e4f88343cf5f6f896994a7d8b830ac0ad6bb74"
links: [global-onboarding, source-development]
reviewed_revision: "git:3410f70938d664269f10f39c50028e57498fd248"
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
