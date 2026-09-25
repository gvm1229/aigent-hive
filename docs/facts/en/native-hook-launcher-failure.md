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
  - "repo:crates/hive-cli/src/policy/configure.rs#sha256:4ffac98a1064f68ed15796c5b1b1893f55fc3ea2bb7bf859851f5f29b384de68"
  - "repo:schemas/host-policy-intent.schema.json#sha256:a37b3e5297c0c460402edd87322b1a1c00c12e39a80ef1877abdc94bdf6b7dda"
  - "repo:tests/conformance/contracts/test_native_policy_hooks.py#sha256:f1365fbdc97d0b4f8e1b18b9dde043a83f609272f48edd485d598301ed04b7bb"
links: [foundation-refactor, project-policy-enforcement]
reviewed_revision: "git:a7359c38fc5510b5d5d34e77217900482f76e712"
status: active
---

# Native Hook Launcher Failure Boundary

The 0.11.0 live-hook investigation found that an absent checker allowed protected edits. Generated PreToolUse launchers now return native denial for missing, failed, or empty-output checkers and discard partial/error output. Healthy host permission responses remain unchanged. Legacy command formats 1 and 2 retain approval bytes. Format 3 prevents outer-shell expansion on Windows and requires a new preview and trust. Validation retains the executable-location binding. This does not prove blocking when the host skips or times out the launcher.
