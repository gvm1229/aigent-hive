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
  - "repo:crates/hive-render/src/activation.rs#sha256:bdc4d8a5dc3a6db95dc2a2530d1df408959a28182211288180d2870c012d2e43"
  - "repo:crates/hive-render/src/lib.rs#sha256:62174ca2ea76cf5c379e1638e3ef2c33cc332f3ae312f03a983711f9abe1a1d5"
  - "repo:crates/hive-update/src/transaction.rs#sha256:09f9a56ab80a841a91a04aba92e772683ee6b8c489c23df1ee3b45a9d1a6bc0c"
links: [test-distribution]
reviewed_revision: "git:e22ff9036b04535dc9a9cc542f08b20143cfe40e"
status: active
---

# Test Fault Isolation

Rust unit tests bind an injected activation failure to the owning test thread in debug and release
builds so parallel update tests cannot consume it. Numeric process-scoped values remain supported
for isolated CLI subprocess conformance tests. Acceptance requires the parser regression
test and repeated parallel `hive-update` suites to pass. This rule was established while
qualifying the user-requested `0.8.0` test distribution.
