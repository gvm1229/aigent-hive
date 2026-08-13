---
schema_version: 1
pair_id: language-consistency
topic_slug: language-consistency
language: en
counterpart: ../ko/language-consistency.md
title: "Response Language Consistency"
summary: "Source agents and consumer harnesses use the selected language unless the user explicitly requests another language for the current response."
tags: [communication, documentation, projection]
aliases: ["language consistency", "mixed-language response"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:9d8adb7c75015fd24df8cb226a16180548c600dc963ee154c0a4af408e9fa48c"
  - "repo:AGENTS.md#sha256:e200aeee1a58fe9224e6e7a0a5a915cde8ac666e4421ea9c6696f0f1cb142fee"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:8a834763d385e30a51f764fdf185bec8cc93a3ecccc22241131c0effc464227c"
  - "repo:harness/template/AGENTS.md.jinja#sha256:9b45162b48b5d6eedcce0d0b6bc2d63613776fc1089a7d2878941897d0d506c5"
links: [global-onboarding, source-development]
reviewed_revision: "git:35e6b79a024350487f823780101a28be24a9f4c7"
status: active
---

# Response Language Consistency

Source-development agents and installed consumer harnesses use the selected
language for every question and response. Only an explicit request for the
current response permits another language; message language alone does not
change the preference. User-scope and project guidance bind the exact `en|ko`
value. Acceptance: an always-loaded source rule plus passing unit,
static-contract, and connected lifecycle tests. Origin: maintainer request to
enforce the installation-selected language across source and product sessions.
