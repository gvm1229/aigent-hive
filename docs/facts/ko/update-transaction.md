---
schema_version: 1
pair_id: update-transaction
topic_slug: update-transaction
language: ko
counterpart: ../en/update-transaction.md
title: "Update transaction"
summary: "Dry run·bounded backup·durable journal·atomic activation."
tags: [recovery, transaction, update]
aliases: ["Safe update transaction"]
sources:
  - "repo:docs/architecture/release-update-trust-boundary.md#sha256:3afa39d749791724d15aaef22619c3bc69eef0286f324f0990b193c6f1617d65"
links: [plugin-update-merge, release-verification]
reviewed_revision: "git:a7be86f2558442c2cec3596abe2f481dd91d268f"
status: active
---

# Update transaction

Verification·deterministic dry run 이후 순서: protected canonical byte snapshot,
첫 mutation 전 durable journal, renderer-owned path만 activation, exact before·after
digest 기반 recovery.
