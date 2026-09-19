---
schema_version: 1
pair_id: update-transaction
topic_slug: update-transaction
language: en
counterpart: ../ko/update-transaction.md
title: "Update Transaction"
summary: "Verified updates use a dry run, bounded backup, durable journal, and atomic activation."
tags: [recovery, transaction, update]
aliases: ["Safe update transaction"]
sources:
  - "repo:crates/hive-update/src/transaction.rs#sha256:f8018544d3c1f69dddfcc2c24d516ce0f259de7b2b759794b8fe3221e9bd41e3"
links: [plugin-update-merge, release-verification]
reviewed_revision: "git:dd63333a702a7a89585d101d2b9d043ebd0987d8"
status: active
---

# Update Transaction

After verification and a deterministic dry run, Hive snapshots protected canonical
bytes, writes a durable journal before mutation, activates only renderer-owned paths,
and rolls back only when exact before or after digests authorize recovery.
Historical full-base sources authenticate every projected file before activation.
