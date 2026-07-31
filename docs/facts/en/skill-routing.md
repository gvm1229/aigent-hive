---
schema_version: 1
pair_id: skill-routing
topic_slug: skill-routing
language: en
counterpart: ../ko/skill-routing.md
title: "Skill Routing and Consent"
summary: "Hive loads one narrow approved Skill after simple-question isolation."
tags: [consent, routing, skill]
aliases: ["Approved Skill routing"]
sources:
  - "repo:docs/architecture/skill-consent.md#sha256:062425d9110c2c52abf9f6b61d06c110f288f415b86706eecbf11439d8ac1c37"
links: [orchestration-ownership, project-onboarding]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Skill Routing and Consent

After simple-question isolation, Hive loads at most one narrowly matching approved
Skill. Optional third-party or generated Skills require an explicit preview and user
approval before activation.
