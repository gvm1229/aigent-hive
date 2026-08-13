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
  - "repo:crates/hive-update/src/transaction.rs#sha256:f5413a85caa8894de16fb749c1b54d57d30834c47b18871e3b2fed51fa29fc91"
links: [plugin-update-merge, release-verification]
reviewed_revision: "git:847d5ad4066e0086faf05219b3ea1f8c21b3d5f3"
status: active
---

# Update Transaction

After verification and a deterministic dry run, Hive snapshots protected canonical
bytes, writes a durable journal before mutation, activates only renderer-owned paths,
and rolls back only when exact before or after digests authorize recovery.
