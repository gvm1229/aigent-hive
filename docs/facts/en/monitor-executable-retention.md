---
schema_version: 1
pair_id: monitor-executable-retention
topic_slug: monitor-executable-retention
language: en
counterpart: ../ko/monitor-executable-retention.md
title: "Monitor executable retention"
summary: "Exact monitor cleanup compares full streams after a no-follow quarantine claim."
tags: [monitor, safety, storage]
aliases: [monitor-executable-retention]
sources:
  - "repo:crates/hive-cli/src/run.rs#sha256:6123c4aa36e88044434ef8bdca5ecfd8b470edcd5cfda1ea7c63171450506dfe"
  - "repo:crates/hive-cli/src/usage_monitor/upgrade.rs#sha256:00b66cc8825ac157550b7dd280cb7599f94bb9172069866df550bb4d8ae2525b"
  - "repo:docs/research/overhaul/OV142-003.md#sha256:d47f74746c332a0ef804eb943106ecd472ed490995c924e7303587dcfd545ae9"
links: [developer-binary-lifecycle]
reviewed_revision: "git:88ecb7a13736634afca601fe354e53c88b0dc58e"
status: active
---

# Monitor executable retention

Monitor cleanup keeps the current and previous executables. Exact removal claims a file in a same-parent quarantine, opens it without following links, compares every expected byte in 64 KiB chunks and requires EOF. Read errors, changed bytes and Windows delete-sharing locks preserve the file. The caller retains its 128 MiB snapshot limit. Windows tests cover 51 MiB files, the update cleanup path and the actual prune CLI. A running Windows copy is restored after delete error 5, reported skipped and leaves no claim directory. These tests do not qualify live monitor replacement or other operating systems.
