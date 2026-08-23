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
  - "repo:crates/hive-cli/src/knowledge_scan.rs#sha256:61bf8cc01a6e0701b89e047ffd0f0118c676a84e8edc7b316cd9c424bbae4f48"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/nested-project-knowledge-scan-0.10.0.md#sha256:09e75e39def220648906afa58722a15a1997ca9013eeeb02f579b8eb4b1aaf8f"
  - "repo:docs/plans/active/release-0.10.0.md#sha256:1b45bd3c62550e7033c664895be32c785404d6c048c5a29856b58e7da0746cdc"
  - "repo:docs/research/ai-learning-hive-application-candidates-2026-08-21.md#sha256:14eb21209b147e7ca9947eae8afb09c059d53aedf353c1802620bf8bf4cc0038"
  - "repo:tests/conformance/integration/test_wiki_cli_e2e.py#sha256:f737ec5b335045a43839360e02c5ed9c2c52d0b9f59394123087fd2063727c12"
links: [knowledge-portability-scan, version-policy]
reviewed_revision: "git:d019b6023bb5b8705da027af638a87b8da3de13d"
status: active
---

# Nested Project Scan in 0.10.0

`0.9.5` is the final `0.9.x` release; no `0.9.6` publication is planned. `SCP10-003` restores
knowledge scanning when a registered project root is below a parent Git repository. `SCP10-003`
is implemented and tested with a nested target and an unchanged foreign-sibling sentinel.
Acceptance requires confinement to the registered root, no sibling access, no global Git
configuration change, and rejection of symlink, junction, or reparse-point escape.
