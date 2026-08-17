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
  - "repo:crates/hive-render/src/lib.rs#sha256:71a3eba58eab1195bc5f6dc5411d81fefd547ab73ef09070479edc0bbe67b091"
  - "repo:crates/hive-update/src/transaction.rs#sha256:3073c4e1ffad221352e29461cbf9bdfc38db704693c0adbaa67052f9ed5875f0"
links: [test-distribution]
reviewed_revision: "git:0fd5ea87fa377dc584dcfa6ad93ae9ee74eb4e97"
status: active
---

# Test Fault Isolation

Rust unit tests bind an injected activation failure to the owning test thread in debug and release
builds so parallel update tests cannot consume it. Numeric process-scoped values remain supported
for isolated CLI subprocess conformance tests. Acceptance requires the parser regression
test and repeated parallel `hive-update` suites to pass. This rule was established while
qualifying the user-requested `0.8.0` test distribution.
