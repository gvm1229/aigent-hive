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
  - "repo:crates/hive-update/src/transaction.rs#sha256:9b6b1f0d01bd69c287783f4d4089e3a580d848a7ff9b97cd13ee3445540edca4"
links: [plugin-update-merge, release-verification]
reviewed_revision: "git:55b0498374de1a10818ab15b2b0eda5fbb4a645d"
status: active
---

# Update Transaction

After verification and a deterministic dry run, Hive snapshots protected canonical
bytes, writes a durable journal before mutation, activates only renderer-owned paths,
and rolls back only when exact before or after digests authorize recovery.
Historical full-base sources authenticate every projected file before activation.
