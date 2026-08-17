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
  - "repo:crates/hive-update/src/transaction.rs#sha256:3073c4e1ffad221352e29461cbf9bdfc38db704693c0adbaa67052f9ed5875f0"
links: [plugin-update-merge, release-verification]
reviewed_revision: "git:0fd5ea87fa377dc584dcfa6ad93ae9ee74eb4e97"
status: active
---

# Update transaction

Verification·deterministic dry run 이후 순서: protected canonical byte snapshot,
첫 mutation 전 durable journal, renderer-owned path만 activation, exact before·after
digest 기반 recovery.
과거 full base 원본: activation 전 모든 projection file exact 인증.
