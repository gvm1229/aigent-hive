---
schema_version: 1
pair_id: orchestration-ownership
topic_slug: orchestration-ownership
language: en
counterpart: ../ko/orchestration-ownership.md
title: "Orchestration Ownership"
summary: "Hive owns provider-neutral iterative control while hosts own model and subagent execution."
tags: [orchestration, ownership]
aliases: ["Orchestration owner"]
sources:
  - "repo:.agents/directives/02-architecture.md#sha256:7b46f6ff26fb81023263a33df7f4363d6fb63a2ac4712e418b86ff36e869247d"
  - "repo:docs/decisions/ADR-0004-orchestration-ownership.md#sha256:0400842448b5e73cedabe1d2eb941abf343a0e1564b2e161c8e54d6677af017e"
  - "repo:docs/decisions/ADR-0015-host-native-skill-composition.md#sha256:c122052f10778e4c0e3c56c9511c2fdb6fc48528ba3d0dba599f91d3be77a5b5"
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:3bdc09d7cc02ced6cf4eb2a4dc4fa5e734653581144d8ab4e57914e21f4bb612"
  - "repo:docs/decisions/ADR-0026-official-cli-extension-points.md#sha256:de58ee344cdb4f4b933a1ba490da5eea13bc076fbdb38c532d1f66bae16e7c66"
links: [judge-verification, model-routed-custom-subagents, product-non-goals, skill-routing, v0-9-skill-suite-plan]
reviewed_revision: "git:3313076aab5bcc9c162b9e9beabc48a777aa5627"
status: active
---

# Orchestration Ownership

ADR-0019 assigns deterministic events, scheduling, leases, receipts, cancel, teams and multi-goal state to Hive; model/subagent execution stays with hosts. No new OMX/OMC dependency. Foreign runs stay read-only; explicit migration creates a new identity, never switches owners in place. Strict iterative/team/multi-goal terminal gates require the authenticated Judge in either mode; ticks/retries never invoke it.

ADR-0026 (2026-11-12) adds official extensions and file inboxes only: interactive Claude, subscription headless Codex/Gemini workers. No CLI key injection, screen capture or headless Claude. ADR-0019 stays unchanged. This decision is not runtime proof.
