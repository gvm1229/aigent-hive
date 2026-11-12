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
  - "repo:crates/hive-cli/src/orchestration.rs#sha256:5617551fbaaffc3495297108a19f05c7aae3e4b0b08b9d56578aaa06bdbcdd84"
  - "repo:crates/hive-cli/src/summon.rs#sha256:6cb0331f2e20b87233d0e8bd9ee8a4dc3f51148e8301867ed0a7333daadfadc9"
  - "repo:crates/hive-core/src/roster.rs#sha256:eaeb59413d1900bcc00ef36e40e35ddd06baff94a8ce129993d622c3991232ea"
  - "repo:docs/decisions/ADR-0026-official-cli-extension-points.md#sha256:ce73d78bda06047d4fe899e1f62f669389978675d668fe37a5a6800bfb76bd10"
links: [hook-inbox-evidence, orchestration-ownership]
reviewed_revision: "git:386bda71844336fe758acc906994991fa5c6f5f0"
status: active
---

# Inert Summon Contract

0.15.0 summon uses schema-checked host observations and binds their exact digest, task and target digests, role, event head and authority. Prepare commits its event/head before publishing task/inbox files. Profiles bind result/cwd placeholders and role-specific sandbox permissions; the user Codex config owns service tier. A profile is not round-trip verified. Preview, refusal and success use ActionResult. Local tests do not qualify real host execution.
