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
  - "repo:crates/hive-render/src/lib.rs#sha256:d19dbe33de8a2244b2f7442da218b4ea1c63e2c7f6ed0dd73a12e88ac03ebf22"
  - "repo:crates/hive-update/src/transaction.rs#sha256:1da5dca3364e6b310ef03cb7fe43c3af9a70e22d5222bc00350a44aa965a6820"
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
