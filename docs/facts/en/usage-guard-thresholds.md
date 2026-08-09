---
schema_version: 1
pair_id: usage-guard-thresholds
topic_slug: usage-guard-thresholds
language: en
counterpart: ../ko/usage-guard-thresholds.md
title: "Usage Guard Thresholds"
summary: "Global setup owns the minimum safety threshold; a registered project may only set an earlier stop."
tags: [guard, project, setup, usage]
aliases: ["Early stop threshold", "Project usage cap"]
sources:
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:87db5fb3f07e5a346d0060eee545bcd22135963c850afbf0e1fd737ba243b1d1"
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:98b58ba14b69581de4035431ed0a970bd3188e4d8dd63a93e993ebc1d4263c55"
links: [global-onboarding, source-usage-guard, usage-sensor-policy]
reviewed_revision: "git:a5dd671385c2a1e09d511fb1de6c737261210df7"
status: active
---

# Usage Guard Thresholds

The global setup threshold is the user-wide safety floor. A registered project can only choose a
higher remaining-usage threshold. The effective threshold is `max(global, project)`: global `20%`,
web `50%`, and game `30%` stop the web project at `50%` and the game project at `30%`. Disabling
the global guard disables every project guard. The planned migration preserves old single-threshold
settings as the global value and rejects invalid or unauthenticated configuration without writes.
