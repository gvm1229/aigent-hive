---
schema_version: 1
pair_id: inert-summon-contract
topic_slug: inert-summon-contract
language: en
counterpart: ../ko/inert-summon-contract.md
title: "Inert Summon Contract"
summary: "Signed Prepare binds host capabilities without enabling worker execution."
tags: [orchestration, roster]
aliases: []
sources:
  - "repo:crates/hive-cli/src/orchestration.rs#sha256:bd252c5454ed7a152bab69ab0f06a1f094ac1c8505ae73d3b7e99000de22a086"
  - "repo:crates/hive-cli/src/summon.rs#sha256:ee1486820a33fe58898f293b7bebb79ca399b6e99b479122f9ee41041af8adf5"
  - "repo:crates/hive-core/src/roster.rs#sha256:383519773089fc0592645e3dbefdb9dfa7f156a944649de9181f6042798ac8a2"
  - "repo:docs/decisions/ADR-0026-official-cli-extension-points.md#sha256:ce73d78bda06047d4fe899e1f62f669389978675d668fe37a5a6800bfb76bd10"
links: [hook-inbox-evidence, orchestration-ownership]
reviewed_revision: "git:386bda71844336fe758acc906994991fa5c6f5f0"
status: active
---

# Inert Summon Contract

0.15.0 summon uses schema-checked host observations and binds their exact digest, task and target digests, role, event head and authority. Prepare commits its event/head before publishing task/inbox files. Profiles bind result/cwd placeholders and role-specific sandbox permissions; the user Codex config owns service tier. A profile is not round-trip verified. Preview, refusal and success use ActionResult. A repeated identical prepared request returns the existing envelope, result and receipts without consuming authority. Local tests do not qualify real host execution.
