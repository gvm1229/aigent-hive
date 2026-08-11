---
schema_version: 1
pair_id: multi-host-user-install
topic_slug: multi-host-user-install
language: en
counterpart: ../ko/multi-host-user-install.md
title: "Multi-host User Install"
summary: "Hive user install and update accept CSV or repeated host selections with aggregate results."
tags: [installation, multi-host, user-setup]
aliases: ["multi-host Hive install", "repeatable host flag"]
sources:
  - "repo:crates/hive-cli/src/main.rs#sha256:f8ea20501bfcc0226a8f720c7e18c5b772389aa423d3796ed8c440d1759bc671"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:d24cc8e55c8706144ac684cb7ccce3bfa9119c4bd0e20a3e6e36222d9d731eea"
  - "repo:docs/hive-install-guide.ko.html#sha256:bf1124b8259e33c56fa3d070a69f1fa1be0f1f3ae8873bb25bd8fe3a5b99418a"
  - "repo:docs/plans/active/multi-host-user-install.md#sha256:048a38d199eb35e838d0772e8162537708f0a006de50614992cd88be49bbb820"
links: [global-onboarding, supported-hosts]
reviewed_revision: "git:565b41f08d02db2308356f1cb5ed35d901337a4b"
status: active
---

# Multi-host User Install

`hive install` and user-scope `hive update` retain single `--host` compatibility and add
`--hosts codex,claude` plus repeated `--host codex --host claude`. Whitespace around commas is
accepted inside one argument; shell input containing that whitespace uses a quoted value such as
`--hosts "codex, claude"`. Host order follows the request.
Duplicate, empty, and unknown selections fail before mutation. Multi-host apply dry-runs every
host first, then executes sequentially. Aggregate JSON reports per-host results; a later failure
reports completed and failed hosts plus retained changed paths.
