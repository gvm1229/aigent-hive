---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: en
counterpart: ../ko/public-skill-identity.md
title: "Public Skill Identity"
summary: "Consumer Skills use short action-oriented names under the aigent-hive plugin namespace, with selected-language descriptors and a fail-closed legacy migration."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:docs/plans/PLAN.md#sha256:4369680b226fd267c1839bdee82b61c9ec2be11a1c8335764f8361f111e8031b"
  - "repo:docs/plans/active/skill-identity-localization.md#sha256:6e5f57ca65dc4e6a94c367dc1ae1e56dbc6d71b22c11a8f1843466dd64aec285"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:90624108d8774fea2ed71efe64a5263cbb14fbe5"
status: active
---

# Public Skill Identity

The proposed consumer contract uses short action-oriented names and the host-provided
`aigent-hive:<name>` namespace. `record-knowledge` records one reviewed durable fact;
`import-repository-knowledge` performs a reviewed bulk repository onboarding. A release migration
must preserve existing selections and local changes or leave the installation unchanged on conflict.
