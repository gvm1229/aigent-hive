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
  - "repo:crates/hive-cli/src/orchestration.rs#sha256:5e20f6c0da237e00c4fa7f354e2e31adc6b378bd5f82797d93771f6e5d745df2"
  - "repo:crates/hive-cli/src/summon.rs#sha256:2bc40ccc70578d5134e740a7c175ac82738aa2b027cf4cd2fcaa983f23cb9b77"
  - "repo:crates/hive-core/src/roster.rs#sha256:d17b1efd3bead532a9cc44cf95d8add4f6ebab53e2305f699637c0daba80ebb7"
  - "repo:docs/decisions/ADR-0026-official-cli-extension-points.md#sha256:ce73d78bda06047d4fe899e1f62f669389978675d668fe37a5a6800bfb76bd10"
links: [hook-inbox-evidence, orchestration-ownership]
reviewed_revision: "git:386bda71844336fe758acc906994991fa5c6f5f0"
status: active
---

# Inert Summon Contract

0.15.0 summon uses schema-checked host observations and binds their exact digest, task and target digests, role, event head and authority. Prepare commits its event/head before publishing task/inbox files. Profiles bind result/cwd placeholders and role-specific sandbox permissions; the user Codex config owns service tier. A profile is not round-trip verified. Preview, refusal and success use ActionResult. Local tests do not qualify real host execution.
