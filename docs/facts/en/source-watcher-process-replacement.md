---
schema_version: 1
pair_id: source-watcher-process-replacement
topic_slug: source-watcher-process-replacement
language: en
counterpart: ../ko/source-watcher-process-replacement.md
title: "Source Watcher Process Replacement"
summary: "A continued Codex thread safely replaces a watcher whose recorded owner process has ended."
tags: [guard, process, source]
aliases: ["Stale watcher owner recovery"]
sources:
  - "repo:tests/conformance/test_source_usage_guard.py#sha256:b173d0c654e77a675f45f5b14e6950ed92ab5230368761ccac9fa472c356ca79"
links: [source-usage-guard, windows-watcher-identity]
reviewed_revision: "git:a1fb6e848117b83354144540df01474e68d25aa8"
status: active
---

# Source Watcher Process Replacement

When a Codex thread continues under a new process, the source gate may retire the
watcher bound to the ended owner's creation identity and start a replacement. It
refuses replacement while the prior owner remains active and verifies the watcher
lease before signaling, preventing PID reuse or unrelated-process termination.
Session bypass remains process-bound and never transfers through this recovery.
