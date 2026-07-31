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
  - "repo:.agents/directives/01-behavior.md#sha256:ea88119052b2208ddcc7fe23c6b8fac640f9ff6b558aa091374ac1da2a1e3cb5"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:33df1458e98a1fa28a6808189b1a8d78b9ade03c93f5796edc881f18fbbec790"
  - "repo:harness/template/AGENTS.md.jinja#sha256:ba73b338622138b0eb68668c55fd14827be6b9d7db2ddfad79b5f481b5d0d045"
links: [global-onboarding, source-development]
reviewed_revision: "git:ed553c9b397c2ce5c0586c28b8aa665bea842c0d"
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
