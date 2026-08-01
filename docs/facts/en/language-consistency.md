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
  - "repo:.agents/directives/01-behavior.md#sha256:a78fc02202dc5c3b934e28924dd86660d297151f4905606dc7a26f2179083eaa"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:e21faccc9dae23d7522de433e345890509ce8d742fa8fe6a375f0892e35713db"
  - "repo:harness/template/AGENTS.md.jinja#sha256:9e5694a62099d262872bd6e1f167d839d9eb3f51c3d6cdfd4884656350cc0ec4"
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
