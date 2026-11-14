---
schema_version: 1
pair_id: test-fault-isolation
topic_slug: test-fault-isolation
language: en
counterpart: ../ko/test-fault-isolation.md
title: "Test Fault Isolation"
summary: "In-process activation faults are scoped to their owning Rust test thread in debug and release test builds."
tags: [release, test, update]
aliases: ["activation fault scope"]
sources:
  - "repo:crates/hive-render/src/activation.rs#sha256:33b2d062ff0344069aff6fe9bf682f3fd1f69043903a3a5755649df729d9e18f"
  - "repo:crates/hive-render/src/lib.rs#sha256:9859ba77fe6044c1da05931adedb5ffc1c9bb523b6953964438afb48a91c26fb"
  - "repo:crates/hive-update/src/transaction.rs#sha256:9b6b1f0d01bd69c287783f4d4089e3a580d848a7ff9b97cd13ee3445540edca4"
links: [test-distribution]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Test Fault Isolation

Rust unit tests bind an injected activation failure to the owning test thread in debug and release
builds so parallel update tests cannot consume it. Numeric process-scoped values remain supported
for isolated CLI subprocess conformance tests. Acceptance requires the parser regression
test and repeated parallel `hive-update` suites to pass. This rule was established while
qualifying the user-requested `0.8.0` test distribution.
