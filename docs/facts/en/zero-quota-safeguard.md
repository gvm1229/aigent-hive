---
schema_version: 1
pair_id: zero-quota-safeguard
topic_slug: zero-quota-safeguard
language: en
counterpart: ../ko/zero-quota-safeguard.md
title: "Default-On Zero Quota Safeguard"
summary: "The 0.12.0 safeguard blocks new Hive work at zero quota despite ordinary disable."
tags: [exhaustion, session, usage, v0-12-0]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:41e5712b85c91de0f0df5505e999b245e60a530d5764f8cf4a35c285b8c4f730"
  - "repo:crates/hive-core/src/usage_guard.rs#sha256:2aaad6ac2375e212caa3bfc5d612f04e2d2c585e1a2560a294d8f4029613e566"
  - "repo:docs/guides/installed-usage-guard.md#sha256:916836c260db60424f19f6540ee5855e490e896f3681b658b07c8d1c063ddc05"
  - "repo:tests/results/public-zero-0.12.0-test.2.json#sha256:d05c55571642fc70b14a8eb63054a4fac6d319c10837ea139a18e0a39cee1597"
links: [installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:16c01d6b233c96ee107914eeca730e3636c5440b"
status: active
---

# Default-On Zero Quota Safeguard

The maintainer requested default-on protection against automatic credit use. In 0.12.0, any observed subscription window or pool at zero blocks new Hive work even when ordinary protection is disabled. Missing, stale or invalid usage also blocks work. A separate confirmed opt-out applies only to the exact current host, conversation and process; other bindings default on. Reset acknowledgement cannot clear exhaustion. Fresh positive usage can clear it, subject to reset protection.

Windows public 0.12.0-test.2 CLI fixtures pass 28 checks; one POSIX link check is skipped. Actual paid-credit prevention, active host interruption and billing changes remain unproved. The current user installation stays 0.11.1.
