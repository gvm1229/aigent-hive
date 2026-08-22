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
  - "repo:docs/decisions/product-release-decisions.md#sha256:3fbe246c3a5b7d2b8ec002d40f73874c056c48ae3a888dede3e40db12eddddac"
  - "repo:docs/plans/active/release-0.10.0.md#sha256:75ece4a12c890f3950d876c96ec605a2a80ebeecfc2ed7255ff3797cc2a33c2e"
  - "repo:docs/research/ai-learning-hive-application-candidates-2026-08-21.md#sha256:14eb21209b147e7ca9947eae8afb09c059d53aedf353c1802620bf8bf4cc0038"
links: [knowledge-portability-scan, version-policy]
reviewed_revision: "git:a0f288b6b962cd5bede27065fa39f708764a621f"
status: active
---

# Nested Project Scan in 0.10.0

`0.9.5` is the final `0.9.x` release; no `0.9.6` publication is planned. `SCP10-003` restores
knowledge scanning when a registered project root is below a parent Git repository. Acceptance
requires confinement to the registered root, no sibling access, no global Git configuration
change, and rejection of symlink, junction, or reparse-point escape.
