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
  - "repo:crates/hive-wiki/src/lib.rs#sha256:414e18a2b7f3576e7d63a7b34aa287ff4e1eb3031c32b5f9aa31ade73170d1ca"
  - "repo:crates/hive-wiki/src/store.rs#sha256:44fdcfac539a78839200855c73b46a391ead6ce5b34514c53b76c5ea762d5c7c"
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:414f31832132c3ad26fae00fb400f972c47edeb4aa9c91c1aaf26c28089edbb9"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:315391aa3b280409c6c19185aff55bcd21af1fb724de89a7007fc84c73a44aa3"
links: [knowledge-storage, project-onboarding]
reviewed_revision: "git:d211300dea66781251306e376e43bf9e798504ef"
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
