---
schema_version: 1
pair_id: native-hook-launcher-failure
topic_slug: native-hook-launcher-failure
language: en
counterpart: ../ko/native-hook-launcher-failure.md
title: "Native Hook Launcher Failure Boundary"
summary: "PreToolUse launchers deny checker execution failures; host loading and timeout remain separate acceptance boundaries."
tags: [hooks, policy, safety]
aliases: []
sources:
  - "repo:crates/hive-cli/src/policy/configure.rs#sha256:c85cea280be13c31da88fd5463eb1504572dd86c7cfb0d2e10ab825a499a0612"
  - "repo:schemas/host-policy-intent.schema.json#sha256:09cbc4e7c9bb5065bb6035356b9a5313708af23baa0f4d2778fbe7920319f495"
  - "repo:tests/conformance/contracts/test_native_policy_hooks.py#sha256:b9471386e86affb501ce3c2b6c7f621c838fe56bef0ac3c32a4b561c36d95082"
links: [foundation-refactor, project-policy-enforcement]
reviewed_revision: "git:80779cb0da9104108c3e5349d6493cd726e9077d"
status: active
---

# Native Hook Launcher Failure Boundary

The 0.11.0 live-hook investigation found that an absent checker allowed protected edits. Generated PreToolUse launchers now return native denial for missing, failed, or empty-output checkers and discard partial/error output. Healthy host permission responses remain unchanged. Legacy command format 1 receipts retain exact approval bytes; format 2 requires a new preview and trust. Validation retains the executable-location binding. This does not prove blocking when the host skips or times out the launcher.
