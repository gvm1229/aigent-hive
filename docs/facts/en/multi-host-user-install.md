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
  - "repo:crates/hive-cli/src/main.rs#sha256:a878391dc02feb77a081ca92519bafbff9d38dbc7cf51824e10ba2ae0c2c132b"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:59e8e730d66ad5ace25e65b3476efeb7dc6b15ac1ba0b27d3666b98beece385c"
  - "repo:docs/archive/plans/foundations/multi-host-user-install.md#sha256:048a38d199eb35e838d0772e8162537708f0a006de50614992cd88be49bbb820"
links: [global-onboarding, supported-hosts]
reviewed_revision: "git:be1a9fc625b4bdb79597be8c3c081381d23e23d3"
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
