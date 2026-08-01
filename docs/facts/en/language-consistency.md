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
  - "repo:.agents/directives/01-behavior.md#sha256:69cad89a5e857e404f6d51106a8688623afd6d3ad1613ddc5a326ab7b998bb30"
  - "repo:AGENTS.md#sha256:14a0d85c5435cebe820cfd9d8fd1271d1fdce73b0ee878f818350b3e1c619fbd"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:c3cc02dcd02afddbd583d51bd02bc113dc283a17e8244587e0bbf832450dd823"
  - "repo:harness/template/AGENTS.md.jinja#sha256:71eeaf7aff5e21b8a7cf764daf6060cb44954f14218370585c3d72a6f25f14c7"
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
