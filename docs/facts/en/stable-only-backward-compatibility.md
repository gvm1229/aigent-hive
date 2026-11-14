---
schema_version: 1
pair_id: stable-only-backward-compatibility
topic_slug: stable-only-backward-compatibility
language: en
counterpart: ../ko/stable-only-backward-compatibility.md
title: "Stable Compatibility and Exact Test Exceptions"
summary: "Stable compatibility requires complete provenance; 0.14.0 adds the exact public 0.13.2-test.2 exception."
tags: [compatibility, provenance, release, v0-12-0]
aliases: []
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:55c9618f6cc2f1e5b18c00c2fc0c8260c6ada6c2020638a0ac2e81d6c0e4e4e0"
  - "repo:crates/hive-render/src/lib.rs#sha256:9859ba77fe6044c1da05931adedb5ffc1c9bb523b6953964438afb48a91c26fb"
  - "repo:docs/decisions/ADR-0025-0.11.2-scope.md#sha256:958df30cdb8c19d4591eed2324587cd0cdb3fb90a30da8ef01daee015efc150e"
  - "repo:tests/results/acceptance-0.12.0-test.3.json#sha256:56a95cce7790c624305121943ac216f2651350f2b9325b8f61a24db566b89304"
  - "repo:tests/results/runs/20261025T093057-ad8ea764ff85.md#sha256:967efbf8264cbd3006be34c738a08c80dc76bc482bbeb4e64141fe95a7ba1369"
links: [source-development, v0-11-2-scope]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Stable Compatibility and Exact Test Exceptions

Stable releases are the default compatibility targets. 0.14.0 adds only the exact public 0.13.2-test.2 package after complete byte authentication. Missing provenance proves no stable origin.

Orireki used official 0.11.0-test.6, but product-only 0.11.0 metadata lost that distinction. The 0.12.0 source preserves compiled test identity in base digests and rejects other test generations without writes. No test.6 exception or forged base exists; installed 0.11.1 and Orireki files were preserved. Public 0.12.0-test.3 passed origin, tamper rejection, historical-test exclusion and stable preservation on Windows, Linux and macOS; actual Orireki migration was excluded.
