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
  - "repo:crates/hive-update/src/transaction.rs#sha256:29bbbc30215315c772c36abcd30e6d45d0c9e9415b952543e692d4718fd8b9b4"
links: [plugin-update-merge, release-verification]
reviewed_revision: "git:847d5ad4066e0086faf05219b3ea1f8c21b3d5f3"
status: active
---

# Update Transaction

After verification and a deterministic dry run, Hive snapshots protected canonical
bytes, writes a durable journal before mutation, activates only renderer-owned paths,
and rolls back only when exact before or after digests authorize recovery.
