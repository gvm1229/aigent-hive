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
  - "repo:crates/hive-update/src/transaction.rs#sha256:dd52b7530d83488b92fc7f7d2a93889ad6c3c7f28bd68d31403da107e38e656d"
links: [plugin-update-merge, release-verification]
reviewed_revision: "git:55b0498374de1a10818ab15b2b0eda5fbb4a645d"
status: active
---

# Update transaction

Verification·deterministic dry run 이후 순서: protected canonical byte snapshot,
첫 mutation 전 durable journal, renderer-owned path만 activation, exact before·after
digest 기반 recovery.
과거 full base 원본: activation 전 모든 projection file exact 인증.
