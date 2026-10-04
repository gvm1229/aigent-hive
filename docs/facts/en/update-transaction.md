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
  - "repo:crates/hive-update/src/transaction.rs#sha256:09f9a56ab80a841a91a04aba92e772683ee6b8c489c23df1ee3b45a9d1a6bc0c"
links: [plugin-update-merge, release-verification]
reviewed_revision: "git:fc288bed8f925b89bfd0ed67b808cfdd0722a70b"
status: active
---

# Update Transaction

After verification and a deterministic dry run, Hive snapshots protected canonical
bytes, writes a durable journal before mutation, activates only renderer-owned paths,
and rolls back only when exact before or after digests authorize recovery.
Historical full-base sources authenticate every projected file before activation.
