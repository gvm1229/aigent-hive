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
  - "repo:tests/conformance/contracts/test_setup_ownership_gates.py#sha256:cd3445a528b2df3d55bebef96476e199a0c7ba88f0509e4f8c71f76debfa5803"
links: [test-fault-isolation]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Windows Namespace Gate Timeout

The foreign-namespace setup gate keeps its ten-second FIFO detection boundary on POSIX.
Windows has no FIFO probe in this test, so loaded CI receives 30 seconds and relies on
exact before/after namespace snapshots. Acceptance requires repeated targeted Windows
runs and the complete gate module to pass. This prevents normal slow setup from being
misclassified as a forbidden namespace read.
