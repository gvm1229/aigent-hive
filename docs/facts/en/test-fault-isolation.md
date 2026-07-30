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
  - "repo:crates/hive-render/src/lib.rs#sha256:46d4bbf77befbc60d21f3787299beeb0f2e3d4acf102fd772c751936c26aa1c4"
  - "repo:crates/hive-update/src/transaction.rs#sha256:d55b9b13726eb812ffdf0e605fe41a24a343157bd41ca175c6750aa6443154ec"
links: [test-distribution]
reviewed_revision: "git:235d5e39a36b4c9d395a65ee4fa7a7ed52515768"
status: active
---

# Test Fault Isolation

Rust unit tests bind an injected activation failure to the owning test thread so
parallel update tests cannot consume it. Numeric process-scoped values remain supported
for isolated CLI subprocess conformance tests. Acceptance requires the parser regression
test and repeated parallel `hive-update` suites to pass. This rule was established while
qualifying the user-requested `0.8.0` test distribution.
