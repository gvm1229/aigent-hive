---
schema_version: 1
pair_id: v0-10-product-scope
topic_slug: v0-10-product-scope
language: en
counterpart: ../ko/v0-10-product-scope.md
title: "Aigent Hive 0.10.0 Product Scope"
summary: "The 0.10.0 scope adds an automatic Korean language core, humanize-kor, and verified im-not-ai upstream packs to the prior graph, workflow, upgrade, and release scope."
tags: [knowledge, language, release, scan, v0-10]
aliases: ["0.10.0 scope"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:ff4dfde9029c9024ab260f0366381e1a9bf1ce9d384a1db46b33d1cd842a5578"
  - "repo:docs/decisions/product-release-decisions.md#sha256:a56419242874c459f08f7575ec0b2b6c2249ac696e0efffb053706dfeb6c9f00"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:12c7ebd3b248e881f8bf9b9cf6da969ef8db2998b096c8668fcc1995c1be39bf"
links: [consumer-session-coordination, graphify-0-10-adoption, hybrid-vector-search-0-10, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:6bb15c4376924d7e3fcbd389daa09550d6477596"
status: active
---

# Aigent Hive 0.10.0 Product Scope

The scope includes relation search, safe upgrade, host-neutral workflows, and an automatic Korean
language core derived from a pinned `im-not-ai` source. Korean responses and Hive-owned writing use
the shared core without a Skill call; `humanize-kor` explicitly edits existing Korean text. Upstream
packs pin version, commit, digests, license, staging, and rollback and never run raw upstream
installers. This post-test product change requires `0.10.0-test.2` or later before stable approval.
