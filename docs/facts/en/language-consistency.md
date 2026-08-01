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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:f790e1c19261cd6367504b074eb5516d8f4486e4bc7316ec9776327847e40272"
  - "repo:harness/template/AGENTS.md.jinja#sha256:65ff8ac38eebfea005dbde07fe1e4cf12148129a050ba1f83296867bcd98a0c5"
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
