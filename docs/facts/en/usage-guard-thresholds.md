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
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:24f84ff3eeb32ba7d5ee5c449d34cf0bb80e300a6123742f22664797402ab219"
links: [global-onboarding, source-usage-guard, usage-sensor-policy]
reviewed_revision: "git:35f5bce71814a3e874fe53a8730024f16013ad46"
status: active
---

# Usage Guard Thresholds

The user selects the global safety floor during setup. A registered project can only choose a
higher remaining-usage threshold. The effective threshold is `max(global, project)`. No project
profile or document provides a fixed percentage. Disabling the global guard disables every project
guard. Migration preserves the old single threshold as the global value and rejects invalid or
unauthenticated configuration without writes. Source development uses the repository source gate and
the same product resolver and project override; no source-only guard Skill, adapter, or threshold
state remains.
