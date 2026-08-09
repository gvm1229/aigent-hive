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
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:5a60d254c760db58049da72530895a981708d549700b02656c7ff51224140f5f"
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:28bc5662cca5ecb730361d7e6519890c0f9db2f800720e9c75d90a854e3d0c80"
links: [global-onboarding, source-usage-guard, usage-sensor-policy]
reviewed_revision: "git:6ed32f63fa3c67bed31164b9d15259f48443341a"
status: active
---

# Usage Guard Thresholds

The user selects the global safety floor during setup. A registered project can only choose a
higher remaining-usage threshold. The effective threshold is `max(global, project)`. No project
profile or document provides a fixed percentage. Disabling the global guard disables every project
guard. Migration preserves the old single threshold as the global value and rejects invalid or
unauthenticated configuration without writes.
