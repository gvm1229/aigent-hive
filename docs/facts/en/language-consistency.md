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
  - "repo:AGENTS.md#sha256:f83b69080a2580ec60feda02ecdb43833b9b43709b2da18dce76c8dd214a0b01"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2960ab32ce7831e993fee381e664d20eb5f673a62f006582a8e601476ccdb0fa"
  - "repo:harness/template/AGENTS.md.jinja#sha256:d706dc6585c1bbaa820d328ebfaae919cd02496adac0acec373ee4d0e37afe56"
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
