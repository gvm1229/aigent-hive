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
  - "repo:crates/hive-render/src/lib.rs#sha256:9c5cf991d1a2e35909cf98b326476bcbfba4e6a8c3141140e0a74a33dc42c782"
  - "repo:crates/hive-update/src/transaction.rs#sha256:236f61f1049854628c5dc572183df02498f67eedefd4ec9058672e554c295042"
links: [test-distribution]
reviewed_revision: "git:847d5ad4066e0086faf05219b3ea1f8c21b3d5f3"
status: active
---

# Test Fault Isolation

Rust unit tests bind an injected activation failure to the owning test thread so
parallel update tests cannot consume it. Numeric process-scoped values remain supported
for isolated CLI subprocess conformance tests. Acceptance requires the parser regression
test and repeated parallel `hive-update` suites to pass. This rule was established while
qualifying the user-requested `0.8.0` test distribution.
