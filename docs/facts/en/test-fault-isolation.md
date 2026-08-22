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
  - "repo:crates/hive-render/src/lib.rs#sha256:340b93226f69b5be1e4c9718e2e6459a5f90725e46355c9ba772d16aa1e5ee5a"
  - "repo:crates/hive-update/src/transaction.rs#sha256:f9ccf1d6ec988d00140708ad83a2912a09301c2bbab9ce97f8f5feac6d79ecd3"
links: [test-distribution]
reviewed_revision: "git:7f6fd5a10898fe4cc9ac59cb4f2035073996d20c"
status: active
---

# Test Fault Isolation

Rust unit tests bind an injected activation failure to the owning test thread in debug and release
builds so parallel update tests cannot consume it. Numeric process-scoped values remain supported
for isolated CLI subprocess conformance tests. Acceptance requires the parser regression
test and repeated parallel `hive-update` suites to pass. This rule was established while
qualifying the user-requested `0.8.0` test distribution.
