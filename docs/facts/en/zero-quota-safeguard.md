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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:c18be9c1e87bb1a34d5ffeffa5aab93b2d620b86723a4ccfc05e804103b16aa8"
  - "repo:crates/hive-core/src/usage_guard.rs#sha256:2aaad6ac2375e212caa3bfc5d612f04e2d2c585e1a2560a294d8f4029613e566"
  - "repo:docs/guides/installed-usage-guard.md#sha256:f1a23a8779e4272fd852313360653f4078bcabd1290c4bebda9c8b0589d077fc"
  - "repo:tests/results/public-zero-0.12.0-test.2.json#sha256:f64d29840b056f157c60f0a5f6556c2e5b62990beb8c9c6bfb3285b4aafd7354"
links: [installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:16c01d6b233c96ee107914eeca730e3636c5440b"
status: active
---

# Default-On Zero Quota Safeguard

The maintainer requested default-on protection against automatic credit use. In 0.12.0, any observed subscription window or pool at zero blocks new Hive work even when ordinary protection is disabled. Missing, stale or invalid usage also blocks work. A separate confirmed opt-out applies only to the exact current host, conversation and process; other bindings default on. Reset acknowledgement cannot clear exhaustion. Fresh positive usage can clear it, subject to reset protection.

Windows public 0.12.0-test.2 CLI fixtures pass 28 checks; one POSIX link check is skipped. Actual paid-credit prevention, active host interruption and billing changes remain unproved. The current user installation stays 0.11.1.
