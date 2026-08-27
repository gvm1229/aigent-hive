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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:88e9456a85123bffb76f1574385b556b9e6b6d9d008862f1c0700daba343ec49"
  - "repo:crates/hive-wiki/src/lib.rs#sha256:3267b6cc3e81bf9af57c5ae267fba3f55cfc21facac25534d8c65c31920c9082"
  - "repo:crates/hive-wiki/src/store.rs#sha256:c8ed85d8dfdbe8d2215cc61f31643d1877f18b035f956146f9ae789de25b200a"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
links: [knowledge-storage, project-onboarding]
reviewed_revision: "git:e5c2c599562121ed3dc43143c16a0b1f063cefa2"
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
