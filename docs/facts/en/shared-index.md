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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:1dccff4126427eee75e24cb440139c4791b1fae34c115cee69d2cb46dd182e87"
  - "repo:crates/hive-wiki/src/lib.rs#sha256:30a94beb01b9496bb6d3e8881cf3a970b46f0edaa669858dd6159dc97894e7bd"
  - "repo:crates/hive-wiki/src/store.rs#sha256:f9f3dcf6627b0c1a2e08a05596f2e027c9a75c9fbbcafb48fbe8d29c000e1481"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
links: [knowledge-storage, project-onboarding]
reviewed_revision: "git:1cdebf8aef9dde963f5aa88a36042186fbf37e80"
status: active
---

# User-root Shared Index

Enabled user and project Markdown feeds one disposable SQLite database under the user
root. Projects do not create independent canonical or derived databases. Shared canonical
mutations hold a user-root operation lock from preparation through canonical writes and the
SQLite rebuild. The separate inner publication lock remains available during that operation.
This prevents another process from observing or replacing the first process's dirty journal.
The concurrent same-page ingest and extraction/integration regression tests are the acceptance
criteria for this serialization path, including Windows CI. Origin: PR #18's Windows Phase 1
conformance jobs exposed the dirty-journal race after the other platform checks passed.
