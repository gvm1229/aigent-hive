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
  - "repo:crates/hive-render/src/lib.rs#sha256:b466679ba789f933f6274a2b7c19bc66851d9e1b6c1925d4249598012e737474"
  - "repo:crates/hive-update/src/transaction.rs#sha256:3f0e96c19402075e4570012f3dcbe7fe4a0e5fe02ac06a9f16775bd44ebe88ff"
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
