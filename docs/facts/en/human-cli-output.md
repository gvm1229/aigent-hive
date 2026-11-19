---
schema_version: 1
pair_id: human-cli-output
topic_slug: human-cli-output
language: en
counterpart: ../ko/human-cli-output.md
title: "Human CLI Output"
summary: "User-facing commands use readable defaults while explicit machine protocols retain their contracts."
tags: [cli, compatibility, output]
aliases: ["human-readable CLI"]
sources:
  - "repo:.agents/directives/02-architecture.md#sha256:6f4b5504814c95be4ef7c7035bd090db945804356dafdaf19944af77eb9c3d3b"
  - "repo:crates/hive-cli/src/human_output.rs#sha256:be0b28414d2c76fc95317bf106576630ccadd5c3f49bafd8cecbe4248b24114a"
  - "repo:crates/hive-cli/src/main.rs#sha256:7fc5b1b31339f6a712d50877bc29722fdedf9fa402ae269d1b52f034d1ffa7e0"
  - "repo:crates/hive-cli/tests/human_output.rs#sha256:8486343385d3a9edf6bb88fc89278437065cccb1c6658a45c0e8030f95818fbc"
links: [inert-summon-contract, usage-threshold-provenance]
reviewed_revision: "git:43dad991f214a9afdc50524cf2b7f32c22ae46a1"
status: active
---

# Human CLI Output

The 0.15.1 source defaults user-facing CLI results to the configured Korean or English. A shared renderer shows the result, changed values, relevant files, recovery and next steps. Explicit --output json keeps supported machine schemas, values and exit codes. Previously unsupported JSON requests remain unsupported, except that 0.15.3 adds JSON results for doctor and check-target. Private worker names, hook, policy hook, usage capture and Discord inbound commands always keep their existing wire protocols. Setup summaries and commands such as usage show no longer require the output option. This describes unreleased source behavior, not an installed or published 0.15.1 release.
