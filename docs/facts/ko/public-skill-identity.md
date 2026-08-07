---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: ko
counterpart: ../en/public-skill-identity.md
title: "Public Skill identity"
summary: "Consumer Skill의 짧은 동작 이름, aigent-hive plugin namespace, 선택 언어 descriptor, fail-closed legacy migration 계약."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:docs/plans/PLAN.md#sha256:4369680b226fd267c1839bdee82b61c9ec2be11a1c8335764f8361f111e8031b"
  - "repo:docs/plans/active/skill-identity-localization.md#sha256:6e5f57ca65dc4e6a94c367dc1ae1e56dbc6d71b22c11a8f1843466dd64aec285"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:90624108d8774fea2ed71efe64a5263cbb14fbe5"
status: active
---

# Public Skill identity

제안 consumer contract: 짧은 동작 이름과 host 제공 `aigent-hive:<name>` namespace. `record-knowledge`:
review된 durable fact 1개 기록. `import-repository-knowledge`: review 기반 repository bulk onboarding.
Release migration 조건: 기존 selection·local change 보존 또는 conflict 시 installation 무변경.
