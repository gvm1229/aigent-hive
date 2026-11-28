---
schema_version: 1
pair_id: adversarial-judge
topic_slug: adversarial-judge
language: ko
counterpart: ../en/adversarial-judge.md
title: "Adversarial Judge Skill"
summary: "Clean-context host-native Judge 요청을 준비하고 기존 authenticated quorum 검증을 재사용하는 0.10.0 명시 단계"
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

- 새 역할: Clean-context request·provider-neutral dispatch envelope 준비
- 실제 launch: Active host 소유
- `hive judge receipt`: Package·assignment·slot·Judge identity·model·effort·verdict digest read-only binding
- 판정: Finding은 diagnostic, 기존 authenticated quorum 통과 뒤에만 acceptance authority
- 금지: Provider 호출·credential·Hive direct process spawn
