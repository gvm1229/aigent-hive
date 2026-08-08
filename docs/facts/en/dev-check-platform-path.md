---
schema_version: 1
pair_id: dev-check-platform-path
topic_slug: dev-check-platform-path
language: en
counterpart: ../ko/dev-check-platform-path.md
title: "Dev-check Platform PATH"
summary: "The pre-push runner composes tool PATH entries without instantiating host-specific pathlib classes."
tags: [development, portability, verification]
aliases: ["dev-check PATH portability"]
sources:
  - "repo:scripts/dev-check.py#sha256:c23a90e8980decca8a4ca290444e0c2e6c721120cf23ecdfe83978152bc2c96f"
links: [release-verification]
reviewed_revision: "git:3feac3e33cd2c7080eb04d1c87e31b354d4dde5c"
status: active
---

# Dev-check Platform PATH

The pre-push runner derives each resolved tool directory with string-based OS path
operations before extending `PATH`. This keeps Windows-mode verification runnable on
non-Windows hosts where constructing a `WindowsPath` is unsupported. Acceptance requires
the `test_dev_check` suite, including the mocked Windows default mode, to pass. The rule
was recorded while qualifying publication of the user-requested Hive-native orchestration plan.
