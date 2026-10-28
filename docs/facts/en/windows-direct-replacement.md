---
schema_version: 1
pair_id: windows-direct-replacement
topic_slug: windows-direct-replacement
language: en
counterpart: ../ko/windows-direct-replacement.md
title: "Windows Direct Replacement"
summary: "Verified update completion and bounded retirement of the exact old image"
tags: [installation, update, windows]
aliases: []
sources:
  - "repo:crates/hive-cli/src/update_activation/direct_replacement.rs#sha256:843c1e751496a72c49ad2c80e54e306d1429b17a05c0405516ebb5e36450762e"
  - "repo:tests/results/runs/20261028T145420-9c52bbfb7ef7.md#sha256:7b3faecc2237ab66cb606251706dbb748dd8b76e4be09d677587827623463e1d"
  - "repo:tests/results/runs/20261028T150434-59ea1195546c.md#sha256:57e83b84a6f8abb170de3c9fdc4cb1bdc8946b1640cc65ab023aac810824dc0b"
links: [interactive-binary-update]
reviewed_revision: "git:9ec3019a07dcfb20506b1c15f64d20504fe96284"
status: active
---

# Windows Direct Replacement

For 0.14.0 completion, core and settings finish before the parent exits. Only the recorded old image waits for exit; pending cleanup never means verified removal. The record binds the prefix identity, versions and both image hashes. An exclusive new file, deleted by Windows on handle close, guards recovery and cleanup. Existing leaves are refused. Resume uses the same record without reinstalling. Hosted Windows normal and interrupted cleanup passed with private images; eight local file tests cover lease conflicts and recovery. Public signing and actual app control need separate evidence.
