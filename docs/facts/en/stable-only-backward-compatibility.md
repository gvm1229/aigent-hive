---
schema_version: 1
pair_id: stable-only-backward-compatibility
topic_slug: stable-only-backward-compatibility
language: en
counterpart: ../ko/stable-only-backward-compatibility.md
title: "Stable-Only Backward Compatibility"
summary: "Stable-Only Backward Compatibility"
tags: [compatibility, provenance, release, v0-12-0]
aliases: []
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:30e453d8375c22b0ec9ec56998fc472dace3fff8cea63ad7ad360d5ff4c19898"
  - "repo:crates/hive-render/src/lib.rs#sha256:62174ca2ea76cf5c379e1638e3ef2c33cc332f3ae312f03a983711f9abe1a1d5"
  - "repo:docs/decisions/ADR-0025-0.11.2-scope.md#sha256:958df30cdb8c19d4591eed2324587cd0cdb3fb90a30da8ef01daee015efc150e"
  - "repo:tests/results/acceptance-0.12.0-test.3.json#sha256:56a95cce7790c624305121943ac216f2651350f2b9325b8f61a24db566b89304"
links: [source-development, v0-11-2-scope]
reviewed_revision: "git:032b1b5083970fa092d85fcd49ea7847c185b15e"
status: active
---

# Stable-Only Backward Compatibility

The maintainer confirmed that test releases are not stable backward-compatibility targets. Orireki used official 0.11.0-test.6 bytes, but its product-only 0.11.0 base metadata lost that distinction. The 0.12.0 source retains an exact compiled test package identity in the base digest and refuses other test generations without mutation. Legacy stable bases still require complete byte authentication; missing provenance does not establish a stable origin. No test.6 compatibility exception or forged consumer base is added. Installed stable 0.11.1 and Orireki consumer files remain unchanged.
- Public test.3 passes origin, tamper rejection, historical-test exclusion and stable preservation on Windows, Linux and macOS; actual Orireki migration is excluded.
