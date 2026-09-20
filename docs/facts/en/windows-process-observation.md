---
schema_version: 1
pair_id: windows-process-observation
topic_slug: windows-process-observation
language: en
counterpart: ../ko/windows-process-observation.md
title: "Windows Process Exit Observation"
summary: "Qualification skips inspection failures only after the held process handle confirms exit."
tags: [acceptance, process, windows]
aliases: []
sources:
  - "repo:scripts/qualify-vector-runtime.py#sha256:2d9160ac0ba4cc751a84f9854e78f59d50c2aa61dc01bb0948a0147e2d8b3179"
  - "repo:tests/results/runs/20260920T205130-e7a95052dc21.md#sha256:7d30c302522f60c5def30463e4115ad9f3b0c9f5652ae82c86f726420b398291"
links: [source-development]
reviewed_revision: "git:6296726906d4b24f82243c4fafae43dbf8f3245f"
status: active
---

# Windows Process Exit Observation

During 0.11.0 public qualification, Windows image queries failed before an exiting process handle became signaled. The observer now waits at most 100ms on the same held handle and skips only confirmed exits. Inspection failure for a live process still fails qualification; it never permits terminating an unverified child. A local comparison of 40 children per version found two errors before the fix and none after. This verifies the tested observer, not all Windows process behavior.
