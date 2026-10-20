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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:836d4ac233d8e719a9cd1b5a09a1deee7111a7c4e208f95f07e277fa3c899b98"
  - "repo:crates/hive-core/src/usage_guard.rs#sha256:2aaad6ac2375e212caa3bfc5d612f04e2d2c585e1a2560a294d8f4029613e566"
  - "repo:docs/guides/installed-usage-guard.md#sha256:735fffd7960d1dca621b1de7bb2fa7f77ef232cda6c75c4165afda76c9f8da50"
  - "repo:tests/results/public-zero-0.12.0-test.2.json#sha256:f64d29840b056f157c60f0a5f6556c2e5b62990beb8c9c6bfb3285b4aafd7354"
links: [installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:ccf59c00e326034d6d12ef9a9c8a52c91d5ff8cc"
status: active
---

# Default-On Zero Quota Safeguard

The maintainer requested default-on protection against automatic credit use. In 0.12.0, any observed subscription window or pool at zero blocks new Hive work even when ordinary protection is disabled. Missing, stale or invalid usage also blocks work. A separate confirmed opt-out applies only to the exact current host, conversation and process; other bindings default on. Reset acknowledgement cannot clear exhaustion. Fresh positive usage can clear it, subject to reset protection.

Windows public 0.12.0-test.2 CLI fixtures pass 28 checks; one POSIX link check is skipped. Actual paid-credit prevention, active host interruption and billing changes remain unproved. That qualification kept the then-active 0.11.1 installation unchanged.
