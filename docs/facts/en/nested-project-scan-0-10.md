---
schema_version: 1
pair_id: nested-project-scan-0-10
topic_slug: nested-project-scan-0-10
language: en
counterpart: ../ko/nested-project-scan-0-10.md
title: "Nested Project Scan in 0.10.0"
summary: "0.9.5 closes the 0.9 release line; safe knowledge scanning for registered projects nested in a parent Git repository moves to 0.10.0."
tags: [knowledge, release, scan, v0-10]
aliases: ["Nested Vault scan"]
sources:
  - "repo:docs/decisions/product-release-decisions.md#sha256:8d73ee9596c08dbf23bab845b1d1c1bed0a86f8d297510b505c9c54dcdee90e1"
  - "repo:docs/plans/active/release-0.10.0.md#sha256:6b7c846152acb98e56c5b5f548d550087ab50eb6bdb4f92d4c9ba0ada79092dc"
  - "repo:docs/research/ai-learning-hive-application-candidates-2026-08-21.md#sha256:14eb21209b147e7ca9947eae8afb09c059d53aedf353c1802620bf8bf4cc0038"
links: [knowledge-portability-scan, version-policy]
reviewed_revision: "git:69697caef2ce83ce939c828e64b55fa349329f82"
status: active
---

# Nested Project Scan in 0.10.0

`0.9.5` is the final `0.9.x` release; no `0.9.6` publication is planned. `SCP10-003` restores
knowledge scanning when a registered project root is below a parent Git repository. Acceptance
requires confinement to the registered root, no sibling access, no global Git configuration
change, and rejection of symlink, junction, or reparse-point escape.
