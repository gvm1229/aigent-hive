---
schema_version: 1
pair_id: shared-index
topic_slug: shared-index
language: en
counterpart: ../ko/shared-index.md
title: "User-root Shared Index"
summary: "One user-root SQLite index projects enabled global and project Markdown."
tags: [index, knowledge]
aliases: ["Shared knowledge index"]
sources:
  - "repo:crates/hive-wiki/src/lib.rs#sha256:292a7ce29540a77026fd99620aac10b35e85f51ee7490e003b19f789c6bf6fd4"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:823da60fecfbe3d02cd5025259184212aee703f6d1e184c4854f15683a769e91"
links: [knowledge-storage, project-onboarding]
reviewed_revision: "git:e72e2f95883ad4503335123d487405d064fb36ac"
status: active
---

# User-root Shared Index

Enabled user and project Markdown feeds one disposable SQLite database under the user
root. Projects do not create independent canonical or derived databases. Shared canonical
mutations publish a persistent dirty marker before writing. When optimistic snapshot
verification detects a concurrent edit, Hive clears that marker before returning the conflict,
so a safe retry remains available. The concurrent same-page ingest regression test is the
acceptance criterion for this recovery path.
