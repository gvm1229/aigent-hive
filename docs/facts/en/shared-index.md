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
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:44401a82ba3bd9f2bc4048876f5480157720bc5fce005a7c0b63f4d960f63bf1"
links: [knowledge-storage, project-onboarding]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# User-root Shared Index

Enabled user and project Markdown feeds one disposable SQLite database under the user
root. Projects do not create independent canonical or derived databases.
