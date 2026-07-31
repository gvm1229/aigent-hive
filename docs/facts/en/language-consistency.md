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
  - "repo:.agents/directives/01-behavior.md#sha256:2532c785b59f23a099b9e4a6eb71798f696dc4b79103600cf7c245582afa9f26"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:c5cb31b7cf39c02be926e38ee529e023aabe45870b84a75b711f4f84c424e282"
  - "repo:harness/template/AGENTS.md.jinja#sha256:e9545c960f609ad7369e2d5e0cc9f48f79fdc7cd20836cf6199f19eb4ca4f301"
links: [global-onboarding, source-development]
reviewed_revision: "git:bd6d9249b8641590269d32deb97d13b2816ba75e"
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
