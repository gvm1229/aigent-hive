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
  - "repo:docs/architecture/release-update-trust-boundary.md#sha256:3afa39d749791724d15aaef22619c3bc69eef0286f324f0990b193c6f1617d65"
links: [plugin-update-merge, release-verification]
reviewed_revision: "git:a7be86f2558442c2cec3596abe2f481dd91d268f"
status: active
---

# Update Transaction

After verification and a deterministic dry run, Hive snapshots protected canonical
bytes, writes a durable journal before mutation, activates only renderer-owned paths,
and rolls back only when exact before or after digests authorize recovery.
