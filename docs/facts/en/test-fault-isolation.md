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
  - "repo:crates/hive-render/src/lib.rs#sha256:db229f0185549b8125115c2600634050675ce05316e4440b8f03657772e29441"
  - "repo:crates/hive-update/src/transaction.rs#sha256:f8018544d3c1f69dddfcc2c24d516ce0f259de7b2b759794b8fe3221e9bd41e3"
links: [test-distribution]
reviewed_revision: "git:0340f8a26d14ebcc134c14e421d605f8022912f8"
status: active
---

# Test Fault Isolation

Rust unit tests bind an injected activation failure to the owning test thread in debug and release
builds so parallel update tests cannot consume it. Numeric process-scoped values remain supported
for isolated CLI subprocess conformance tests. Acceptance requires the parser regression
test and repeated parallel `hive-update` suites to pass. This rule was established while
qualifying the user-requested `0.8.0` test distribution.
