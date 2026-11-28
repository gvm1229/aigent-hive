---
schema_version: 1
pair_id: adversarial-judge
topic_slug: adversarial-judge
language: en
counterpart: ../ko/adversarial-judge.md
title: "Adversarial Judge Skill"
summary: "0.10.0 adds an explicit adversarial-judge step that prepares a clean-context host-native Judge request and reuses authenticated Hive quorum verification."
tags: [judge, skills, v0-10]
aliases: ["Adversarial review"]
sources:
  - "repo:crates/hive-cli/src/judge.rs#sha256:8663d98c5f70ac4e15b5b25266f3166a8184ee4ef064c6df6b4d4f3d688a3c94"
  - "repo:docs/archive/plans/adversarial-judge-0.10.0.md#sha256:952b369d86a293d96c61f200379fac63590d70e66600bb6d43aea65bf4a130b8"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:71a8c4e0f75fdc3f99aa7f4d1c4e2ace759af640dbc4dd013174639e71a3f01b"
  - "repo:docs/decisions/product-release-decisions.md#sha256:3620f38dc575abdd65508175d3a33b7e998b957d70b05c8c807337eafa5a0321"
  - "repo:harness/skills/judge/references/adversarial.md#sha256:0894b287519b907460640a0e3eb74f10e684864892d0af85b3a12bd79f160a64"
  - "repo:schemas/adversarial-judge-host-receipt.schema.json#sha256:9a4758a7fa9c9e81a0da1969370913ddf7bb53dd80db2b3fb75c65cac21551ba"
links: [judge-verification, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:c06e1d0e8077c4fb0a20bb209c2580570454f354"
status: active
---

# Adversarial Judge Skill

`adversarial-judge` prepares a clean-context request and provider-neutral dispatch envelope. The
active host launches the separate Judge. Read-only `hive judge receipt` binds the host launch and
result to the exact package, assignment, slot, Judge identity, model, effort, and verdict digest.
Findings remain diagnostic until the existing authenticated quorum authorizes acceptance. Hive
never calls a provider or spawns a process.
