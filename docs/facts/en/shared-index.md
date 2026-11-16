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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:9d4a9694f24b3a7d01c34c7eab0c1172434edc8005128880634680c187d2bc69"
  - "repo:crates/hive-wiki/src/lib.rs#sha256:ea24603d3912833478398bce9ca3a9a217334651dfbd38bbd7e553740b9e16c0"
  - "repo:crates/hive-wiki/src/store.rs#sha256:812c302031f36694ad109479698f9d89b125392dba55468c0131e52d45f47b44"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
links: [knowledge-storage, project-onboarding]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
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
