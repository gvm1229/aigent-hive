---
schema_version: 1
pair_id: skill-routing
topic_slug: skill-routing
language: en
counterpart: ../ko/skill-routing.md
title: "Skill Routing and Consent"
summary: "Narrow intent routing, optional Skill approval, and safe source-consumer Skill reuse."
tags: [consent, routing, skills]
aliases: ["Skill routing"]
sources:
  - "repo:AGENTS.md#sha256:8293c7e01a78bbf6106fc6ee9cca9748171ba2361c5003883ad11faa4a81b396"
  - "repo:docs/architecture/skill-consent.md#sha256:062425d9110c2c52abf9f6b61d06c110f288f415b86706eecbf11439d8ac1c37"
  - "repo:harness/skills/catalog.yml#sha256:43ac874922e77cd461576c213a51397757e282ca55a9d94f923e1b13bf4435cf"
links: [knowledge, plugin-lifecycle, workflow]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# Skill Routing and Consent

Hive routes explicit task intent through narrow Skill descriptions and typed catalog entries. It
does not add a duplicate model-based prompt classifier. A self-contained simple question remains
on the direct-answer path without loading project memory, unrelated Skills, or orchestration.

Optional third-party or generated Skills require an exact approval payload bound to name, immutable
source, revision, content digest, requested capabilities, approved capabilities, and approval time.
Any identity or capability change requires fresh approval. Invalid consent keeps the Skill inert.

Hive-owned Skills may move between consumer and source development only after scope, safety,
consent, and conformance review. Shared source stays canonical under `harness/skills/`; an exact
source projection may live under `.agents/skills/`. Installed consumer state, user knowledge, and
runtime data are never treated as source material.
