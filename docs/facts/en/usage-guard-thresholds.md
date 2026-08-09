---
schema_version: 1
pair_id: usage-guard-thresholds
topic_slug: usage-guard-thresholds
language: en
counterpart: ../ko/usage-guard-thresholds.md
title: "Usage Guard Thresholds"
summary: "The user selects the global safety threshold; a registered project may only set an earlier stop."
tags: [guard, project, setup, usage]
aliases: ["Early stop threshold", "Project usage cap"]
sources:
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:2150681617bd1c2273780f0796609f27fc4815418428c0743ef11b88245deb38"
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:fed21b2de4b06f8034974ea611ce0afb2c0b09244a57c16238a2a1c662a131f8"
links: [global-onboarding, source-usage-guard, usage-sensor-policy]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
status: active
---

# Usage Guard Thresholds

The user selects the global safety floor during setup. A registered project can only choose a
higher remaining-usage threshold. The effective threshold is `max(global, project)`. No project
profile or document provides a fixed percentage. Disabling the global guard disables every project
guard. Migration preserves the old single threshold as the global value and rejects invalid or
unauthenticated configuration without writes. Source development uses this same product guard,
resolver, and project override; no source-only guard Skill, adapter, or threshold state remains
after the planned migration.
