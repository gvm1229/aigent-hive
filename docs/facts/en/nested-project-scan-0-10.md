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
  - "repo:crates/hive-cli/src/knowledge_scan.rs#sha256:250b9f581271389c06b4740a16ccc90761faeeeeda238b901c1195205ef7d605"
  - "repo:docs/decisions/product-release-decisions.md#sha256:3620f38dc575abdd65508175d3a33b7e998b957d70b05c8c807337eafa5a0321"
  - "repo:docs/plans/active/nested-project-knowledge-scan-0.10.0.md#sha256:09e75e39def220648906afa58722a15a1997ca9013eeeb02f579b8eb4b1aaf8f"
  - "repo:docs/plans/active/release-0.10.0.md#sha256:2b8007e0cbf5a0f89ebb654ee7f6b44a1b203eee905205fe7ea90629941e4cad"
  - "repo:docs/research/ai-learning-hive-application-candidates-2026-08-21.md#sha256:14eb21209b147e7ca9947eae8afb09c059d53aedf353c1802620bf8bf4cc0038"
  - "repo:tests/conformance/integration/test_wiki_cli_e2e.py#sha256:430fb068e1e87dba1bec696bce9db71383bf0b3d40184c233d1be7a1b7629cab"
links: [knowledge-portability-scan, version-policy]
reviewed_revision: "git:e22ff9036b04535dc9a9cc542f08b20143cfe40e"
status: active
---

# Nested Project Scan in 0.10.0

`0.9.5` is the final `0.9.x` release; no `0.9.6` publication is planned. `SCP10-003` restores
knowledge scanning when a registered project root is below a parent Git repository. `SCP10-003`
is implemented and tested with a nested target and an unchanged foreign-sibling sentinel.
Acceptance requires confinement to the registered root, no sibling access, no global Git
configuration change, and rejection of symlink, junction, or reparse-point escape.
