---
schema_version: 1
pair_id: language-consistency
topic_slug: language-consistency
language: en
counterpart: ../ko/language-consistency.md
title: "Response Language Consistency"
summary: "English responses use ASD-STE100. Korean responses use Korean unless an exact English literal is necessary."
tags: [communication, documentation, harness, language]
aliases: ["Korean response", "controlled English", "language consistency"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:6587e8fa5aa274f2c981ad28c062d3c8c388e351440c04663a50122570986976"
  - "repo:.agents/directives/08-human-documentation-style.md#sha256:c5e0f385ab0bdb17979eee241bc77ad8531d5fb4e29198654bb28b9185164884"
  - "repo:harness/project-bases/0.9.0/AGENTS.md.template#sha256:0d3c02bcd6269879b635b83d7ed22a0e4a9fd6e1b15f75a0b2a1e496f808e57d"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:640be28ec7f75444a52544b0d36c45363696dcbd0281f9c5aabd0768d185784e"
  - "repo:harness/template/AGENTS.md.jinja#sha256:63e361ae2218f00f6a22f5e192c25a5c3bcddc21d51f61006f74a0459b636a38"
links: [global-onboarding, source-development]
reviewed_revision: "git:721f888e97222d8c32e67eb5c546dc070189090a"
status: active
---

# Response Language Consistency

Source agents and consumer harnesses use the selected language. Only an
explicit request for the current response can change it.

English responses use ASD-STE100 Simplified Technical English. Use short
direct sentences, concrete verbs, and one main point per sentence. Do not use
idiom, filler, vague pronouns, stacked clauses, or unnecessary synonyms.

Korean responses use Korean vocabulary and sentence structure. Keep English
only for a necessary exact literal or technical term. Do not use mixed
Korean-English compounds or emphasis-only English parentheticals. Translate
meaning, not English word order. Acceptance: source and consumer contracts,
four matching user-setup projections, and rendered lifecycle tests.
