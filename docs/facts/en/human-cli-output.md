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
  - "repo:crates/hive-cli/src/human_output.rs#sha256:9deda4082d077779be5017e8810eadd8e3338f9a9b0264baa907ea62c250099b"
  - "repo:crates/hive-cli/src/main.rs#sha256:c4c2fda2321700dea40ebcc070d936bfab399df8efe4febb3f2875cd19d687ac"
  - "repo:crates/hive-cli/tests/human_output.rs#sha256:00747bd54b01d1358a91633026103f2b4b28c272962abeb0e8a9ee89c89acbe5"
links: [inert-summon-contract, usage-threshold-provenance]
reviewed_revision: "git:43dad991f214a9afdc50524cf2b7f32c22ae46a1"
status: active
---

# Human CLI Output

The 0.15.1 source defaults user-facing CLI results to the configured Korean or English. A shared renderer shows the result, changed values, relevant files, recovery and next steps. Explicit --output json keeps supported machine schemas, values and exit codes. Previously unsupported JSON requests remain unsupported. Private worker names, hook, policy hook, usage capture and Discord inbound commands always keep their existing wire protocols. Setup summaries and commands such as usage show no longer require the output option. This describes unreleased source behavior, not an installed or published 0.15.1 release.
