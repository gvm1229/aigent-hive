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
  - "repo:crates/hive-cli/src/main.rs#sha256:edc9e588f4303932f8323ec26f58f00481da7585ae7334c4b8d5048959aa7e20"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:6ee84b034e6a23171889dc33e8be8f839594edd080168ad7601e8c5fd9e5c9cc"
  - "repo:docs/archive/plans/foundations/multi-host-user-install.md#sha256:048a38d199eb35e838d0772e8162537708f0a006de50614992cd88be49bbb820"
  - "repo:docs/hive-install-guide.ko.html#sha256:31a2c507fb0b2d266c012ca62cfd91a69b9e6847deaf8eaa1a3abe455ea83d85"
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
