---
schema_version: 1
pair_id: artifact-boundaries
topic_slug: artifact-boundaries
language: en
counterpart: ../ko/artifact-boundaries.md
title: "Artifact Boundaries"
summary: "Source workspace, release bundle, and installed harness are separate artifacts."
tags: [artifact, boundary]
aliases: ["Three artifact classes"]
sources:
  - "repo:docs/decisions/ADR-0001-source-release-installed-boundary.md#sha256:51850d51887f4d2cd4759e562aedee458398463e2b219cb94ca7b4540ad5bab7"
links: [crate-ownership, product-non-goals]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Artifact Boundaries

Hive keeps the source workspace, reproducible release bundle, and installed consumer
harness physically and logically separate. Source-development directives never ship
as consumer instructions.
