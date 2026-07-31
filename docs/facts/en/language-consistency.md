---
schema_version: 1
pair_id: language-consistency
topic_slug: language-consistency
language: en
counterpart: ../ko/language-consistency.md
title: "Response Language Consistency"
summary: "Source agents and consumer harnesses keep each question and response in the selected language."
tags: [communication, documentation, projection]
aliases: ["language consistency", "mixed-language response"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:d59f86031a7bb6f889eeaa00598794fdd2f73375da7d03cdb6a5b49d4884dc0f"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:0ed886384328d10f394f0f2f8fb6f1deed69908af026ecaa17e1e75e17b39a3a"
  - "repo:harness/template/AGENTS.md.jinja#sha256:6198d9b0380ee4e46d44a6aab9ea759c0080690e3353a9309da1a12c5b1939c2"
links: [global-onboarding, source-development]
reviewed_revision: "git:8c190672e3f08ade9bdf985016bcf7b00fa157a1"
status: active
---

# Response Language Consistency

Source-development agents and installed consumer harnesses keep the selected
language consistent across each question and response. Korean retains English
only for proper nouns, product or package names, commands, identifiers, paths,
schema keys, exact UI labels, and terms without a clear Korean equivalent.
English passages remain English except for exact Korean names, literals,
quotations, or user-requested preserved text. Acceptance: matching source,
user-scope, and project guidance plus passing projection and language regression
tests. Origin: maintainer request to prevent avoidable mixed-language prose from
obscuring action lists.
