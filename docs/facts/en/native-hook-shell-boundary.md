---
schema_version: 1
pair_id: native-hook-shell-boundary
topic_slug: native-hook-shell-boundary
language: en
counterpart: ../ko/native-hook-shell-boundary.md
title: "Native File Hook Shell Boundary"
summary: "The Codex file hook does not enforce shell writes."
tags: [acceptance, codex, hooks]
aliases: []
sources:
  - "repo:tests/results/runs/20260920T084743-7e741284a754.md#sha256:825cc29760132bc4c76286242ba585eb312342e6abafc42a0d834f16940ebef2"
links: [codex-file-hook-acceptance]
reviewed_revision: "git:47d631d8cbe1f21186225d5760a2854f75ec1239"
status: active
---

# Native File Hook Shell Boundary

During approved isolated acceptance, Windows Codex 0.155.0-alpha.9.2 executed a shell append to a synthetic protected backup file. The command exited zero and an independent byte check confirmed the change. The controller restored the exact baseline. The registered apply_patch matcher does not cover this shell route. This is evidence of unsupported protection, not successful enforcement or evidence about other hosts. The user requested completion of the remaining host acceptance criteria.
