---
schema_version: 1
pair_id: zero-quota-safeguard
topic_slug: zero-quota-safeguard
language: en
counterpart: ../ko/zero-quota-safeguard.md
title: "Default-On Zero Quota Safeguard"
summary: "The 0.13.2 source accepts a zero floor and qualifies Windows interruption while retaining independent exhaustion protection."
tags: [exhaustion, session, usage, v0-12-0]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:6a8b7909e224bde9bcf45ea91e8267219b6a7b1a4546f3b601705db94d144a88"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:2bfa879fe9bc0d682e6562c3d323d2400daf80478140a3a1d1fc5aa7b09621ed"
  - "repo:crates/hive-core/src/usage_guard.rs#sha256:3c1f90928debba27a9bb5080e04d4db7b59b15aa32a5dd88b121eb10712624bf"
  - "repo:docs/guides/installed-usage-guard.md#sha256:735fffd7960d1dca621b1de7bb2fa7f77ef232cda6c75c4165afda76c9f8da50"
  - "repo:docs/research/usage-compatibility-0.13.2.md#sha256:6096c88095e0ba39fc78dfe959356c2f8d988c2115e3b5519bd55ccf3041464d"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:332849449782b95d9d5819316151e0e8991477fa2cc56ba5df568dcc35f4c237"
  - "repo:tests/results/public-zero-0.12.0-test.2.json#sha256:f64d29840b056f157c60f0a5f6556c2e5b62990beb8c9c6bfb3285b4aafd7354"
  - "repo:tests/results/usage-compatibility-live-0.13.2.json#sha256:ae9ad865350a8e411d89a3bf77d7e6a8433d78302247b7bba51fdde444e918a9"
links: [installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:f6bec784d94c76ef79cda8ff28e60a0b05765805"
status: active
---

# Default-On Zero Quota Safeguard

The maintainer requested default-on protection against automatic credit use. In 0.12.0, any observed subscription window or pool at zero blocks new Hive work even when ordinary protection is disabled. Missing, stale or invalid usage also blocks work. A separate confirmed opt-out applies only to the exact current host, conversation and process; other bindings default on. Reset acknowledgement cannot clear exhaustion. Fresh positive usage can clear it, subject to reset protection.

Windows public 0.12.0-test.2 CLI fixtures pass 28 checks; one POSIX link check is skipped. That 0.12.0 qualification did not prove paid-credit prevention, active host interruption or billing changes. That qualification kept the then-active 0.11.1 installation unchanged.

The 0.13.2 source accepts floors from 0 through 99. Zero remains enabled protection: positive quota may run, while exhausted windows block regardless of credit balance. Live zero receipts are scoped to each active turn, so a new turn cannot inherit a prior exhaustion stop. Windows Codex 26.1016.8120.0 controlled-observation tests confirmed tool interruption in 1,161ms and a new turn stop in 758ms. The app survived. No additional model tools are allowed after observed exhaustion. Real provider-reset interruption, AI-child interruption in this acceptance and zero billing loss remain unverified.
