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
  - "repo:crates/hive-update/src/transaction.rs#sha256:a49e1c5e8596f0aa4cd640bcf75c537c19bc90f613e119d11abc927fdbc86e0d"
links: [plugin-update-merge, release-verification]
reviewed_revision: "git:7f6fd5a10898fe4cc9ac59cb4f2035073996d20c"
status: active
---

# Update Transaction

After verification and a deterministic dry run, Hive snapshots protected canonical
bytes, writes a durable journal before mutation, activates only renderer-owned paths,
and rolls back only when exact before or after digests authorize recovery.
Historical full-base sources authenticate every projected file before activation.
