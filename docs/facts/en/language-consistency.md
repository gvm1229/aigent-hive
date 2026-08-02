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
  - "repo:.agents/directives/01-behavior.md#sha256:d1e3d4cbc89c962bfae66b5a9c135562bd962fa0d8a3765ad2d150e4a9e41195"
  - "repo:AGENTS.md#sha256:14a0d85c5435cebe820cfd9d8fd1271d1fdce73b0ee878f818350b3e1c619fbd"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:9963a01fd3f6e86eee944e3fa331437e283616322c0f15bfb33254e31efff9bd"
  - "repo:harness/template/AGENTS.md.jinja#sha256:c53d41177ef323c50041c8e02928fd1db9904188c22d652d5a80dfbd454228e5"
links: [global-onboarding, source-development]
reviewed_revision: "git:19eda4d7ef87fe3122c14c455df07758c3dc6ff1"
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
