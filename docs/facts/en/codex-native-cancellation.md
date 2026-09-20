---
schema_version: 1
pair_id: codex-native-cancellation
topic_slug: codex-native-cancellation
language: en
counterpart: ../ko/codex-native-cancellation.md
title: "Codex Native Cancellation Boundary"
summary: "Manual Stop and same-task continuation were observed in Windows Codex."
tags: [acceptance, cancellation, codex]
aliases: []
sources:
  - "repo:docs/research/codex-native-cancellation-2026-09-20.md#sha256:e227bccdcf6da8693275ab7a6c3f074404c0c94e3a20c06c12895eb5eb8bc49d"
links: [codex-file-hook-acceptance]
reviewed_revision: "git:a42459c1d8b6701aefcd7ad527c430184ce9949f"
status: active
---

# Codex Native Cancellation Boundary

Windows Codex 0.155.0-alpha.9.2 recorded interrupted after the user pressed Stop. A subsequent request in the same task completed while the original turn stayed interrupted. Host state inspection proves this manual cancellation and continuation. It does not prove Hive automatic interruption, 15-second monitoring, Stop-hook execution, or other hosts. The resume response reported a clock lookup; its tool item was not exposed by the read API.
