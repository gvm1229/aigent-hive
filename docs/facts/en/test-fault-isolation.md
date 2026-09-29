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
  - "repo:crates/hive-render/src/lib.rs#sha256:388ee4c000ff000246d04a626d3b2018d284bf79f85e645d6145a2bdac99254e"
  - "repo:crates/hive-update/src/transaction.rs#sha256:09f9a56ab80a841a91a04aba92e772683ee6b8c489c23df1ee3b45a9d1a6bc0c"
links: [test-distribution]
reviewed_revision: "git:fc288bed8f925b89bfd0ed67b808cfdd0722a70b"
status: active
---

# Test Fault Isolation

Rust unit tests bind an injected activation failure to the owning test thread in debug and release
builds so parallel update tests cannot consume it. Numeric process-scoped values remain supported
for isolated CLI subprocess conformance tests. Acceptance requires the parser regression
test and repeated parallel `hive-update` suites to pass. This rule was established while
qualifying the user-requested `0.8.0` test distribution.
