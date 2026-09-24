---
schema_version: 1
pair_id: directive-context-recovery
topic_slug: directive-context-recovery
language: en
counterpart: ../ko/directive-context-recovery.md
title: "Directive Recovery Across Compactions"
summary: "Selective recovery and mutation checks are the proposed response to repeated context loss."
tags: [context, hooks, policy]
aliases: []
sources:
  - "repo:docs/plans/directive-context-recovery-0.11.0.md#sha256:6854b75f36c91d6ccb2fb8e71bfcd7ca0fbebaaa2281418fb790d26a63afc9ff"
links: [project-policy-enforcement]
reviewed_revision: "git:7539f44b947b1fe0c90ea3d5c083022c99c9e341"
status: active
---

# Directive Recovery Across Compactions

The user requests directive retention across repeated compactions with minimal token overhead and unnecessary enforcement. The reviewed 0.11.0 proposal restores a small core after compaction, loads relevant details on demand, and checks deterministic rules at mutation boundaries. File freshness does not prove context presence. Research and planning are complete; implementation and live compaction qualification remain unperformed.
