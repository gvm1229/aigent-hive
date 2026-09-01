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
  - "repo:crates/hive-cli/src/main.rs#sha256:024500782daa35d5ab3a6df26a443bf0e4c0653a2a2c19caaa2f1b2a7836cdb6"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:7a5c873834ba9a77e6efdedc60a5eed953fa40102dfcf88c084db5b591f465c3"
  - "repo:docs/archive/plans/foundations/multi-host-user-install.md#sha256:048a38d199eb35e838d0772e8162537708f0a006de50614992cd88be49bbb820"
  - "repo:docs/hive-install-guide.ko.html#sha256:fb6dbc2e5752fe9693c7ac791841b2e2f9995ef03a2d5c3ef4ce07b6f068800c"
links: [global-onboarding, supported-hosts]
reviewed_revision: "git:f91816a46d44d57929cb0b580ca32ff4caa95053"
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
