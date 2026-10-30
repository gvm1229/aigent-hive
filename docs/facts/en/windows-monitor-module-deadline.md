---
schema_version: 1
pair_id: windows-monitor-module-deadline
topic_slug: windows-monitor-module-deadline
language: en
counterpart: ../ko/windows-monitor-module-deadline.md
title: "Windows monitor module deadline"
summary: "Fixed module paths keep cold monitor requests within their deadline."
tags: [isolation, powershell, usage-guard, windows]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_monitor/windows.ps1#sha256:d3908b27c3e339a7ff99d2ebd6008c27055abbb26f3f43cfc04b49b9e0406b13"
  - "repo:tests/results/runs/20261028T094859-a3ae3ce2a902.md#sha256:36254ee6ccc20f7f990f9d97a161657a9c7d0b6a4d11ef70e8e19850ae034a89"
  - "repo:tests/results/runs/20261028T095333-28c178240686.md#sha256:5c11e35200273275f2c5e814ff418a20e7a0b0bf550916ab14fd79ee3affb34c"
links: [windows-powershell-module-isolation]
reviewed_revision: "git:5e4c55012c03f15ccc0f8cad2aaa8f3403871b1c"
status: active
---

# Windows monitor module deadline

For the 0.14.0 monitor-start investigation, isolated Windows Server 2025 took 28.703 seconds with automatic module discovery and 0.875 seconds with fixed-module imports. The fixed script imports Utility/CimCmdlets from PSHOME, Appx for inspection and ScheduledTasks for task operations. Request deadlines and ownership checks remain unchanged. Evidence covers read-only diagnosis and scheduler-stub checks, not full activation, live app control or public-release acceptance.
