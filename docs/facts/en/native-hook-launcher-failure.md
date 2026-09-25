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
  - "repo:crates/hive-cli/src/policy/configure.rs#sha256:bab9fc28b65e564c28630f54bbb4991dff734d0185da72b59ee9f54800163875"
  - "repo:schemas/host-policy-intent.schema.json#sha256:0e1877e37080235b1ebc905b02ba7b6b7c245cce7c110b46cb94b6c9b8aacb4d"
  - "repo:tests/conformance/contracts/test_native_policy_hooks.py#sha256:f1365fbdc97d0b4f8e1b18b9dde043a83f609272f48edd485d598301ed04b7bb"
links: [foundation-refactor, project-policy-enforcement]
reviewed_revision: "git:a7359c38fc5510b5d5d34e77217900482f76e712"
status: active
---

# Native Hook Launcher Failure Boundary

The 0.11.0 live-hook investigation found that an absent checker allowed protected edits. Generated PreToolUse launchers now return native denial for missing, failed, or empty-output checkers and discard partial/error output. Healthy host permission responses remain unchanged. Legacy command formats 1 and 2 retain approval bytes. Format 3 prevents outer-shell expansion on Windows and requires a new preview and trust. Validation retains the executable-location binding. This does not prove blocking when the host skips or times out the launcher.
