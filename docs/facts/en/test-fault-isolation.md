---
schema_version: 1
pair_id: test-fault-isolation
topic_slug: test-fault-isolation
language: en
counterpart: ../ko/test-fault-isolation.md
title: "Test Fault Isolation"
summary: "In-process activation faults are scoped to their owning Rust test thread."
tags: [release, test, update]
aliases: ["activation fault scope"]
sources:
  - "repo:crates/hive-render/src/lib.rs#sha256:23ec7f468b5ce965b52ebeb5a900bf7468b22edd7fb7b9c89afec345f6bea359"
  - "repo:crates/hive-update/src/transaction.rs#sha256:ac0f1ce8924f939006c6a94f80795bc605053e9cd815a19d3b345890150b01d0"
links: [test-distribution]
reviewed_revision: "git:33f365d3dbb1af51333a6dbb1834ce437a932ea0"
status: active
---

# Test Fault Isolation

Rust unit tests bind an injected activation failure to the owning test thread so
parallel update tests cannot consume it. Numeric process-scoped values remain supported
for isolated CLI subprocess conformance tests. Acceptance requires the parser regression
test and repeated parallel `hive-update` suites to pass. This rule was established while
qualifying the user-requested `0.8.0` test distribution.
