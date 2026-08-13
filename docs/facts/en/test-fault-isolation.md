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
  - "repo:crates/hive-render/src/lib.rs#sha256:9d5ae48c8c77e11cc59db83c53a387d2e85329e4508b66558b40c55a419f0534"
  - "repo:crates/hive-update/src/transaction.rs#sha256:f5413a85caa8894de16fb749c1b54d57d30834c47b18871e3b2fed51fa29fc91"
links: [test-distribution]
reviewed_revision: "git:9a125333ed070140b3773462d895684cba62fe6b"
status: active
---

# Test Fault Isolation

Rust unit tests bind an injected activation failure to the owning test thread in debug and release
builds so parallel update tests cannot consume it. Numeric process-scoped values remain supported
for isolated CLI subprocess conformance tests. Acceptance requires the parser regression
test and repeated parallel `hive-update` suites to pass. This rule was established while
qualifying the user-requested `0.8.0` test distribution.
