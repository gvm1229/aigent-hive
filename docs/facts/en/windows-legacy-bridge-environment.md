---
schema_version: 1
pair_id: windows-legacy-bridge-environment
topic_slug: windows-legacy-bridge-environment
language: en
counterpart: ../ko/windows-legacy-bridge-environment.md
title: "Windows Legacy Bridge Environment"
summary: "Remove owned transport variables before the compiler starts."
tags: [compatibility, isolation, update, windows]
aliases: []
sources:
  - "repo:crates/hive-cli/src/update_bridge/windows.rs#sha256:6300accf43181a9f1e1746029db78a33ed23fa870c0b6987dcbeff05fe0f9b76"
  - "repo:tests/results/runs/20261028T055203-acfe0611bc2e.md#sha256:7f713d57b290777472bf108acb3ff91832c2beb03e741278d01741816df11d1f"
  - "repo:tests/results/runs/20261028T055306-db9e1b405451.md#sha256:7e94b4a0014c46552089127c875c0681acd4445759ad5d151cf186ea581fc06f"
links: []
reviewed_revision: "git:2b9e88e113352ec50c743d69b83b19d7798f2be8"
status: active
---

# Windows Legacy Bridge Environment

The 0.14.0 first-update work found that Windows CodeDOM rejects inherited environments above 65,535 bytes. Read each owned HIVE_UPDATE_BRIDGE_n chunk into memory, then remove that exact variable in the helper before Add-Type starts its compiler. Preserve other variables and parent authentication.

The isolated failure and Windows transport tests cover this boundary. They do not qualify a public release or real app/monitor control.
