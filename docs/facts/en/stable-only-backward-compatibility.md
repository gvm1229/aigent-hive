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
  - "repo:docs/decisions/ADR-0025-0.11.2-scope.md#sha256:72f2989faf99a78eb09b3f84fe6feb8842e3004b575c7267e4048a2213ee3214"
links: [source-development, v0-11-2-scope]
reviewed_revision: "git:8aefd94546860569fcb5337105381f065b3f887a"
status: active
---

# Stable-Only Backward Compatibility

The maintainer confirmed that test releases are not stable backward-compatibility targets. Orireki used official 0.11.0-test.6 bytes, but its product-only 0.11.0 base metadata lost that distinction. The 0.12.0 source retains an exact compiled test package identity in the base digest and refuses other test generations without mutation. Legacy stable bases still require complete byte authentication; missing provenance does not establish a stable origin. No test.6 compatibility exception or forged consumer base is added. Installed stable 0.11.1 and Orireki consumer files remain unchanged.
