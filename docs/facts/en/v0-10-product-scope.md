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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:b88eaf08d187d6f83cfac8b9e3a186791f08b71d0d5287f5dafe4d2e7aaa8151"
  - "repo:docs/decisions/product-release-decisions.md#sha256:a56419242874c459f08f7575ec0b2b6c2249ac696e0efffb053706dfeb6c9f00"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:12c7ebd3b248e881f8bf9b9cf6da969ef8db2998b096c8668fcc1995c1be39bf"
links: [consumer-session-coordination, graphify-0-10-adoption, hybrid-vector-search-0-10, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:e8bbe0529513df56e73f84cf5797bb334f4184ec"
status: active
---

# Aigent Hive 0.10.0 Product Scope

The scope includes relation search, safe upgrade, host-neutral workflows, an automatic Korean
language core, and reopened vector qualification. Vector work separates duplicate synthetic data
from unique chunks and requires resumable embedding, end-to-end latency, scope isolation, atomic
generation, rollback, and three-platform evidence. No engine is preselected. The Korean core and
any gate-passing optional hybrid adapter require `0.10.0-test.2` or later before stable approval.
