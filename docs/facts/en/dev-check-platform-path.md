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
  - "repo:scripts/dev-check.py#sha256:6ba74f6a9b9f262b11046e3f2003285839b447e55b2f03aeafc62f054e1690ba"
links: [release-verification]
reviewed_revision: "git:39569b7a2a7c67f8ab19010db8c4df32da470f86"
status: active
---

# Dev-check Platform PATH

The pre-push runner derives each resolved tool directory with string-based OS path
operations before extending `PATH`. This keeps Windows-mode verification runnable on
non-Windows hosts where constructing a `WindowsPath` is unsupported. Acceptance requires
the `test_dev_check` suite, including the mocked Windows default mode, to pass. The rule
was recorded while qualifying publication of the user-requested Hive-native orchestration plan.
