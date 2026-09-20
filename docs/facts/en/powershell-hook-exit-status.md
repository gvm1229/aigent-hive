---
schema_version: 1
pair_id: powershell-hook-exit-status
topic_slug: powershell-hook-exit-status
language: en
counterpart: ../ko/powershell-hook-exit-status.md
title: "PowerShell Hook Exit Status Boundary"
summary: "An outer PowerShell can map a diagnostic exit 2 to exit 1."
tags: [hooks, powershell, verification]
aliases: []
sources:
  - "repo:tests/results/runs/20260920T190500-9123781755ba.md#sha256:31be3f7db845673adcd8b04623aeb9531d60e1bca2b8963afefa18cfd7d9e416"
links: [native-hook-launcher-failure]
reviewed_revision: "git:ff082a7d25848a82d4cf081f1d8e7701a2da618e"
status: active
---

# PowerShell Hook Exit Status Boundary

The same encoded Windows diagnostic returned exit 2 through cmd and exit 1 through an outer PowerShell, with the same exit-2 diagnostic on stderr. This deterministic reproduction matches the reported CLI failure but does not identify the actual host shell independently. Verify the host-visible status instead of assuming child exit codes survive shell layers. JSON denial and process failure are distinct acceptance cases.
