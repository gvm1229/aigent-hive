---
schema_version: 1
pair_id: usage-default-off-scope
topic_slug: usage-default-off-scope
language: en
counterpart: ../ko/usage-default-off-scope.md
title: "Usage Guard and Interactive CLI Scope"
summary: "The 0.14.0 CLI implementation has scoped evidence; full OS, model and public-file acceptance remains open."
tags: [guard, planning, usage]
aliases: []
sources:
  - "repo:crates/hive-core/src/preferences.rs#sha256:838005e60b3255422c2ee143d76e66ef3ef877d7f3a8ff196eb446c0dfdc888d"
  - "repo:docs/decisions/interactive-cli-0.14.0.md#sha256:db25524e2ae9fede4fb70c9a7662a2f6c05ef87fbe7a2eaaff356377951731d2"
  - "repo:docs/plans/0.14.0-implementation.md#sha256:685d7db95252c69c24f49f393b2e53350cc3506bd5543e21346bb3a26c8bed5b"
  - "repo:docs/research/cli-usage-settings-0.14.0.md#sha256:5252d4ad330030b7ca44c7fbb7cc482d3df23acdd6a6ad4172a9854994da5615"
  - "repo:docs/research/cli-user-settings-0.14.0.md#sha256:029f15d8f36dbc94ad6e63444e3ba8ad22ae8ef1566a0f2a6080d77bb110f636"
  - "repo:docs/research/interactive-cli-0.14.0.md#sha256:ffd6dd3695c3ef9128e006d2191d93460fa177856b2c90c6621ce3c7ddc4297a"
links: [usage-guard-stop-layers]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Usage Guard and Interactive CLI Scope

0.14.0 covers local user/project setup, questions during the first update from public 0.13.1, and setting-origin records. New automatic stops default off. Existing active or unknown choices receive CLI review; unattended updates preserve choices and defer questions. Removal requires preview and confirmation while preserving user data; full data deletion is excluded. Direct CLI setup makes zero model calls. AI input reduction and Luna qualification require actual measurement. Source implementation and scoped CLI evidence exist; complete OS, model and public-file acceptance remains open. The next public test is 0.14.0-test.1.
