---
schema_version: 1
pair_id: windows-namespace-gate-timeout
topic_slug: windows-namespace-gate-timeout
language: en
counterpart: ../ko/windows-namespace-gate-timeout.md
title: "Windows Namespace Gate Timeout"
summary: "The Windows foreign-namespace gate allows loaded CI setup up to 30 seconds."
tags: [ci, test, windows]
aliases: ["Windows setup timeout"]
sources:
  - "repo:tests/conformance/test_phase1_stage2_gates.py#sha256:cd8b65a09de2c964c787a0302f046f971fc47d3ae38084ee77bbbafed8225efc"
links: [test-fault-isolation]
reviewed_revision: "git:5f50bcde8a96782e95023486232992cc41b9abc1"
status: active
---

# Windows Namespace Gate Timeout

The foreign-namespace setup gate keeps its two-second FIFO detection boundary on POSIX.
Windows has no FIFO probe in this test, so loaded CI receives 30 seconds and relies on
exact before/after namespace snapshots. Acceptance requires repeated targeted Windows
runs and the complete gate module to pass. This prevents normal slow setup from being
misclassified as a forbidden namespace read.
