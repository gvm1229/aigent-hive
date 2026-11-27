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
  - "repo:.agents/directives/02-architecture.md#sha256:1a2382dc375a5983f994a9180cda03355e8b79c69cb9a3195c0d4353331ef4ed"
  - "repo:crates/hive-cli/src/human_output.rs#sha256:bf38581607b1cf16698c8b1a13ef6028dd42e5c963187299e3e452ce6ce77bc9"
  - "repo:crates/hive-cli/src/main.rs#sha256:80eef72b676bab2bceaed789f57fb9181f89df18f74aeebc5d22a62c54f7ac77"
  - "repo:crates/hive-cli/tests/human_output.rs#sha256:ed87bc2cfbf8ccfac0a3e4a8aeb779903dfdca2f0728f7647244e6bec650f65b"
links: [inert-summon-contract, usage-threshold-provenance]
reviewed_revision: "git:43dad991f214a9afdc50524cf2b7f32c22ae46a1"
status: active
---

# Human CLI Output

The 0.15.1 source defaults user-facing CLI results to the configured Korean or English. A shared renderer shows the result, changed values, relevant files, recovery and next steps. Explicit --output json keeps supported machine schemas, values and exit codes. Previously unsupported JSON requests remain unsupported, except that 0.15.3 adds JSON results for doctor and check-target. Private worker names, hook, policy hook, usage capture and Discord inbound commands always keep their existing wire protocols. Setup summaries and commands such as usage show no longer require the output option. This describes unreleased source behavior, not an installed or published 0.15.1 release.
