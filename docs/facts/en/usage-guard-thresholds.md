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
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:4b99d1f046ff56eeb9102b99dec4e88226ca2cdfa4947bb233c9a5c541a19172"
links: [global-onboarding, installed-usage-guard, usage-sensor-policy]
reviewed_revision: "git:ced55f4d0b18b259c9b43e0f9622b6d617a65737"
status: active
---

# Usage Guard Thresholds

The user selects the global safety floor during setup. A registered project can only choose a
higher remaining-usage threshold. The effective threshold is `max(global, project)`. No project
profile or document provides a fixed percentage. Disabling the global guard disables every project
guard. Migration preserves the old single threshold as the global value and rejects invalid or
unauthenticated configuration without writes. The Hive source workspace uses the global threshold
without a project override. Non-Hive folders do not inherit the global threshold and cannot mutate
it through a project-target request. The maintainer's current global threshold is 5% remaining.
