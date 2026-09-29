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
  - "repo:crates/hive-update/src/transaction.rs#sha256:09f9a56ab80a841a91a04aba92e772683ee6b8c489c23df1ee3b45a9d1a6bc0c"
links: [plugin-update-merge, release-verification]
reviewed_revision: "git:fc288bed8f925b89bfd0ed67b808cfdd0722a70b"
status: active
---

# Update transaction

Verification·deterministic dry run 이후 순서: protected canonical byte snapshot,
첫 mutation 전 durable journal, renderer-owned path만 activation, exact before·after
digest 기반 recovery.
과거 full base 원본: activation 전 모든 projection file exact 인증.
