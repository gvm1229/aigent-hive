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
  - "repo:.agents/directives/01-behavior.md#sha256:24e61b7fd37bc1b9e0a73933547d5b369b9ca2cdde6c9adc10ba29bd23d50143"
  - "repo:AGENTS.md#sha256:8a5f2a661c03a43976d7e88bce188a07a8e17db569b82ae76e83c3807914b30a"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:3f634fae4465c6c868462a29c502c698b69a2ed7f28a40249c10a26b9804c21f"
  - "repo:harness/template/AGENTS.md.jinja#sha256:070d97440343d699565448c239efb55c905df79119df289525d41edc6e81581f"
links: [global-onboarding, source-development]
reviewed_revision: "git:33f365d3dbb1af51333a6dbb1834ce437a932ea0"
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
