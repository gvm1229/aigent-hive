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
  - "repo:scripts/dev-check.py#sha256:8078227723acce5d1f0795e55a0088e972a15facd8dbf624ab1a3cc0baadfa60"
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
