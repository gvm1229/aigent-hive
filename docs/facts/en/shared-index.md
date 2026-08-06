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
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dc4ddb908ecef82f197b0055cea3adb36602f0b924f8a285add60c1d9b7f7ec7"
links: [knowledge-storage, project-onboarding]
reviewed_revision: "git:d211300dea66781251306e376e43bf9e798504ef"
status: active
---

# User-root Shared Index

Enabled user and project Markdown feeds one disposable SQLite database under the user
root. Projects do not create independent canonical or derived databases. Shared canonical
mutations publish a persistent dirty marker before writing. When optimistic snapshot
verification detects a concurrent edit, Hive clears that marker before returning the conflict,
so a safe retry remains available. The concurrent same-page ingest regression test is the
acceptance criterion for this recovery path.
