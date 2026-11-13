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
  - "repo:crates/hive-cli/src/human_output.rs#sha256:61c71804508ee28e92b23508f47f3c002190dd6e500e478b28d946a56122c030"
  - "repo:crates/hive-cli/src/main.rs#sha256:a68944d85c5808e5090bba2b54977d6e484f7acdccfbbbd5acc2ab20ba75f7cd"
  - "repo:crates/hive-cli/tests/human_output.rs#sha256:6814fec5d42b5c7358bfca51cbbea1263a204673d572d7e759da285948f00111"
links: [inert-summon-contract, usage-threshold-provenance]
reviewed_revision: "git:43dad991f214a9afdc50524cf2b7f32c22ae46a1"
status: active
---

# Human CLI Output

The 0.15.1 source defaults user-facing CLI results to the configured Korean or English. A shared renderer shows the result, changed values, relevant files, recovery and next steps. Explicit --output json keeps supported machine schemas, values and exit codes. Previously unsupported JSON requests remain unsupported. Private worker names and native stdin-JSON hooks explicitly select their existing wire protocols. Setup summaries and commands such as usage show no longer require the output option. This describes unreleased source behavior, not an installed or published 0.15.1 release.
