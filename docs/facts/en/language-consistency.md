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
  - "repo:.agents/directives/01-behavior.md#sha256:53b809a61225b5d860c37c8c61459960d26306aaf19e550fe79ce50984eebf9e"
  - "repo:.agents/directives/08-human-documentation-style.md#sha256:f1c700565caf1c448cfa0a7d58db549d5c3d466b264737233fe255c67663acd6"
  - "repo:harness/project-bases/0.9.0/AGENTS.md.template#sha256:1aefece59d56d610227b64cfcfff8c634e47202f8e224916b248a8e8ecd9de51"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:f5d9b13356fb64171213e98b41045955760247c6f5e1ce420c991afe450063de"
  - "repo:harness/template/AGENTS.md.jinja#sha256:3d14ecded34d198d08e5aba138239e933fc2670888db4bc3c4637984572076e6"
links: [global-onboarding, source-development]
reviewed_revision: "git:3410f70938d664269f10f39c50028e57498fd248"
status: active
---

# Response Language Consistency

Source agents and consumer harnesses use the selected language. Only an
explicit request for the current response can change it.

English responses use ASD-STE100 Simplified Technical English. Use short
direct sentences, concrete verbs, and one main point per sentence. Do not use
idiom, filler, vague pronouns, stacked clauses, or unnecessary synonyms.

Korean responses use Korean vocabulary and sentence structure. Do not use
replaceable English words, mixed Korean-English compounds, technical-sounding
English, or emphasis-only English parentheticals. The contracts pair prohibited
forms with natural Korean replacements. Acceptance: source and consumer
contracts, four matching user-setup projections, and rendered lifecycle tests.
