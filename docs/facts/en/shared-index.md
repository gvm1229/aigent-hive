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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:db6b864c91ed2e96f16481fe9bf8818ebb6955de1760e41f753a424a3c310ea5"
  - "repo:crates/hive-wiki/src/lib.rs#sha256:7fa7b0f9b2b366fd3707ad7f25999b8a5c018ab2ddb9c134fd75969636352adc"
  - "repo:crates/hive-wiki/src/store.rs#sha256:5b3bc23d06794f76d40f879644e621a8132f5c177f052c2e314205cd7e5c80bd"
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
