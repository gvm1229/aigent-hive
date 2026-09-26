---
schema_version: 1
pair_id: codex-file-hook-acceptance
topic_slug: codex-file-hook-acceptance
language: en
counterpart: ../ko/codex-file-hook-acceptance.md
title: "Codex File Hook Acceptance Boundary"
summary: "Format 3 was exercised in Windows Codex for healthy and missing-checker file-edit behavior."
tags: [acceptance, codex, hooks]
aliases: []
sources:
  - "repo:tests/results/runs/20260920T030341-ee67c1f8f88f.md#sha256:dd85153bd78b4a6d6b8ceb9dbfed4038c21d60b3269129c2f6d076b4cf21dac2"
  - "repo:tests/results/runs/20260920T030607-c02090c781e6.md#sha256:9d18a93c6ebc187150fe692ca491818f257ede74107a47b6f1f80b986dfd53a6"
links: [native-hook-launcher-failure]
reviewed_revision: "git:2c8ed3149a2bcf31dd9068f4fc9ac0d87dca2ab7"
status: active
---

# Codex File Hook Acceptance Boundary

After the user confirmed format-3 trust, Windows Codex 0.155.0-alpha.9.2 allowed one normal edit and denied one protected edit. With the checker temporarily absent, both edits were denied and their bytes stayed unchanged. The controller restored the executable and fixtures. This proves those four attempts with the frozen 0.11.0 checker, not other AI hosts, unloaded hooks, host timeouts, other tool families, or native turn cancellation.
