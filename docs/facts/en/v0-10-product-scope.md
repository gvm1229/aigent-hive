---
schema_version: 1
pair_id: v0-10-product-scope
topic_slug: v0-10-product-scope
language: en
counterpart: ../ko/v0-10-product-scope.md
title: "Aigent Hive 0.10.0 Product Scope"
summary: "The 0.10.0 scope adds a Korean language core and reopens vector qualification with safe embedding, isolation, rollback, and conditional one-engine adoption."
tags: [knowledge, language, release, scan, v0-10]
aliases: ["0.10.0 scope"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:921f2847dacea259c29b9f6c8cbb2c4f7c090429e04771ec240d49eb1ccfbb72"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:f93dba421edc980e3a9ca8b5a8ce2ee978806094ae360203371948a27bcadaec"
links: [consumer-session-coordination, graphify-0-10-adoption, hybrid-vector-search-0-10, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:eaed3203ce3fea062acab325a9ce0892348aff02"
status: active
---

# Aigent Hive 0.10.0 Product Scope

The scope includes relation search, safe upgrade, host-neutral workflows, and the automatic Korean
language core. Vector requalification ended in `defer` because the unique 50,000-chunk build failed
the ten-minute gate; no product dependency exists. The Korean core, `humanize-kor`, pinned pack,
bounded host adapters, and rollback are implemented. They require `0.10.0-test.2` or later and
three-platform acceptance before stable approval.
