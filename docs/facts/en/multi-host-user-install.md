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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:6e7acffee805aea9462b9f350be5d99b9220f719894a19d384e4a5f4756822d9"
  - "repo:docs/hive-install-guide.ko.html#sha256:1e7113e8769a0f2cc6e0c1b8848144ff51be8a46b0605e48454a7359fed24e93"
  - "repo:docs/plans/active/multi-host-user-install.md#sha256:be31244b9e9a39b4cba728765245a02cd0ad2868612827e9d74e7bba3bff0ca8"
links: [global-onboarding, supported-hosts]
reviewed_revision: "git:6bab86b8421b50154967cec080c430ab05704bd8"
status: active
---

# Multi-host User Install

`hive install` and user-scope `hive update` retain single `--host` compatibility and add
`--hosts codex,claude` plus repeated `--host codex --host claude`. Host order follows the request.
Duplicate, empty, and unknown selections fail before mutation. Multi-host apply dry-runs every
host first, then executes sequentially. Aggregate JSON reports per-host results; a later failure
reports completed and failed hosts plus retained changed paths.
