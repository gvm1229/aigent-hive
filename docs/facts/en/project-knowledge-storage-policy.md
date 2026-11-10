---
schema_version: 1
pair_id: project-knowledge-storage-policy
topic_slug: project-knowledge-storage-policy
language: en
counterpart: ../ko/project-knowledge-storage-policy.md
title: "Project knowledge storage and access"
summary: "Visibility and promotion settings do not grant global read-only storage."
tags: [index, knowledge, policy]
aliases: []
sources:
  - "repo:crates/hive-cli/src/knowledge/remember.rs#sha256:b3f259173a92bf050ca26df5b462330ea4ce9d96bb9bca431b6a54700ddfea36"
  - "repo:crates/hive-wiki/src/shared.rs#sha256:4e65063d1f7d0dca6db2361f465136c892d8d357fa641c8337f29ba68a356965"
  - "repo:docs/plans/details/project-local-global-readonly/design.md#sha256:18defa9a2f898ab073eebdf6345a2e64a449619879f2b4f46af074708b335a12"
links: [knowledge-cross-project-access, shared-index]
reviewed_revision: "git:dd581c30013516a6649e74e3c0d80bf7d46cb278"
status: active
---

# Project storage boundary

In 0.14.0, project-private claims and derived search data use the user-root store.
Empty promotion categories do not grant read-only access to that store.
An initialized alternate user root works, but does not provide policy-bound reads
from a second store. The requested local-write investigation separates storage,
visibility, promotion and write authority. Acceptance needs filesystem observations,
not only unchanged final hashes. Scope and limits:
[diagnosis](../../research/project-local-global-readonly-0.14.0/README.md).

Plan boundary: explicit opt-in only; preserve existing read/write behavior and defaults.
Common attachment repairs are a separate workstream.
