---
schema_version: 1
pair_id: automatic-user-projection-refresh
topic_slug: automatic-user-projection-refresh
language: en
counterpart: ../ko/automatic-user-projection-refresh.md
title: "Automatic User Projection Refresh"
summary: "The 0.9.5 plan requires a bare binary update to refresh only the authenticated saved user-install scope."
tags: [installation, migration, projection, update]
aliases: ["Post-update projection refresh"]
sources:
  - "repo:crates/hive-cli/src/update_activation.rs#sha256:88a29477d0b59f362df545687e6939267545980d9f78eb7e10d8f3322f81a94c"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b8bb2ace9f509cf8f48cf703971069e7ca73ada3704a8c7dc18adfd03a27e9ef"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:73d95136c28b6742d76d7aca0874144808500168a67fc97accdee9bc2b387481"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:9170c884c9c96d99abcea1f5ab96a4a3a62541be"
status: active
---

# Automatic User Projection Refresh

The `0.9.5` plan requires bare `hive update` to refresh the user projection only after an
authenticated binary replacement. It derives the semantic host scope from valid saved setup and
per-host install manifests, never from a default host or untrusted command history. Missing or
invalid scope produces a clearly reported binary-only outcome with no projection mutation.
