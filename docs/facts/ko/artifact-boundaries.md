---
schema_version: 1
pair_id: artifact-boundaries
topic_slug: artifact-boundaries
language: ko
counterpart: ../en/artifact-boundaries.md
title: "Artifact 경계"
summary: "Source workspace, release bundle, installed harness의 분리."
tags: [artifact, boundary]
aliases: ["세 artifact class"]
sources:
  - "repo:docs/decisions/ADR-0001-source-release-installed-boundary.md#sha256:51850d51887f4d2cd4759e562aedee458398463e2b219cb94ca7b4540ad5bab7"
links: [crate-ownership, product-non-goals]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Artifact 경계

물리·논리 분리 대상: source workspace, reproducible release bundle, installed consumer
harness. Source-development directive의 consumer instruction 출하 금지.
