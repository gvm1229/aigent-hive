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
  - "repo:scripts/dev-check.py#sha256:2518cca41040548e7f62a060c8bb8e0a77e880ad7b21a6ebd599892f4a0b5ece"
links: [release-verification]
reviewed_revision: "git:c06e1d0e8077c4fb0a20bb209c2580570454f354"
status: active
---

# Dev-check Platform PATH

The pre-push runner derives each resolved tool directory with string-based OS path
operations before extending `PATH`. This keeps Windows-mode verification runnable on
non-Windows hosts where constructing a `WindowsPath` is unsupported. Acceptance requires
the `test_dev_check` suite, including the mocked Windows default mode, to pass. The rule
was recorded while qualifying publication of the user-requested Hive-native orchestration plan.
