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
  - "repo:crates/hive-render/src/lib.rs#sha256:54c93f6fdf51beda50d73eba8e3ea0a06e2441f69f93a8b40440a9d2fc37d767"
  - "repo:crates/hive-update/src/transaction.rs#sha256:12687aaeb13ec6266060d9b0e3549829a6e0470eb361161d78e9e0bdb289caaa"
links: [test-distribution]
reviewed_revision: "git:47d4663f1e1f263276f9ce54b7c69a3ff95d2170"
status: active
---

# Test Fault Isolation

Rust unit tests bind an injected activation failure to the owning test thread in debug and release
builds so parallel update tests cannot consume it. Numeric process-scoped values remain supported
for isolated CLI subprocess conformance tests. Acceptance requires the parser regression
test and repeated parallel `hive-update` suites to pass. This rule was established while
qualifying the user-requested `0.8.0` test distribution.
