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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:eb9876cbd3af9428f4bd09216421b5d741e1e506dfc93ef478e5305c1c513193"
  - "repo:crates/hive-wiki/src/lib.rs#sha256:70b39f3dda7bc2d3eae393b4705524d27755ffed39deb141cac879d11b837ca7"
  - "repo:crates/hive-wiki/src/store.rs#sha256:39f62b339764e470446c61bfb392b2f8637908738261c8fe5bc9b711da0bb40d"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
links: [knowledge-storage, project-onboarding]
reviewed_revision: "git:35f5bce71814a3e874fe53a8730024f16013ad46"
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
