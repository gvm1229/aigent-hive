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
  - "repo:crates/hive-render/src/activation.rs#sha256:17682ee8e6b20f3ba51ea8e6aae3aeb3df546aae0e7059070b3107df85ebea59"
  - "repo:crates/hive-render/src/lib.rs#sha256:4b07b57fefd87c090dc12db47a756f0f5556545f6316c879bf8e0a67f53190b4"
  - "repo:crates/hive-update/src/transaction.rs#sha256:dd52b7530d83488b92fc7f7d2a93889ad6c3c7f28bd68d31403da107e38e656d"
links: [test-distribution]
reviewed_revision: "git:be1a9fc625b4bdb79597be8c3c081381d23e23d3"
status: active
---

# Test Fault Isolation

Rust unit tests bind an injected activation failure to the owning test thread in debug and release
builds so parallel update tests cannot consume it. Numeric process-scoped values remain supported
for isolated CLI subprocess conformance tests. Acceptance requires the parser regression
test and repeated parallel `hive-update` suites to pass. This rule was established while
qualifying the user-requested `0.8.0` test distribution.
