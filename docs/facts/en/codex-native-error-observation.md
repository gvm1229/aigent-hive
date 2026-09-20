---
schema_version: 1
pair_id: codex-native-error-observation
topic_slug: codex-native-error-observation
language: en
counterpart: ../ko/codex-native-error-observation.md
title: "Codex Native Hook Error Observation"
summary: "Invocation markers distinguish JSON denial from synthetic hook failures."
tags: [acceptance, codex, hooks]
aliases: []
sources:
  - "repo:tests/results/runs/20260920T190935-ea2a51ed1c04.md#sha256:71d74c6c4f1b4ef40415b51a1b8d092fb7440e34fec807245ead029518c09091"
links: [codex-file-hook-acceptance, powershell-hook-exit-status]
reviewed_revision: "git:9051144140b4b892721b7ce8a8300e35c6ba7932"
status: active
---

# Codex Native Hook Error Observation

Windows Codex 0.155.0-alpha.9.2 ran six synthetic diagnostic cases, each confirmed by one matching marker. JSON denial blocked the edit and preserved bytes. Cases producing child exit 1/2/3, delayed output beyond the native timeout, or malformed JSON allowed edits. Outer-shell exit conversion is separate. These are additive diagnostic hooks on normal files, not injected faults in the production Hive checker. Original definitions and all six baselines were restored.
