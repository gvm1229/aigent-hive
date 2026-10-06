---
schema_version: 1
pair_id: installed-usage-guard
topic_slug: installed-usage-guard
language: en
counterpart: ../ko/installed-usage-guard.md
title: "Installed Guard Target Boundary"
summary: "The installed guard applies only to configured Hive projects and the Hive source workspace; non-Hive folders remain entirely inactive."
tags: [guard, source, usage]
aliases: ["Installed usage policy"]
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:41e5712b85c91de0f0df5505e999b245e60a530d5764f8cf4a35c285b8c4f730"
  - "repo:docs/guides/installed-usage-guard.md#sha256:916836c260db60424f19f6540ee5855e490e896f3681b658b07c8d1c063ddc05"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:ad412a7b2aa6bba927f5f841e0e5161f4fb60488"
status: active
---

# Installed Guard Target Boundary

- Installed product: sole usage-guard implementation
- Configured Hive project: `max(global, project)` and project-local session state
- Aigent Hive source: global threshold·user-root runtime·source `.hive/` files `0건`
- Non-Hive folder: enforcement·threshold mutation·session override·halt·runtime `0건`; setup-free Skills available
- Session control: explicit configured target only; unrelated malformed graph `CURRENT.md` preserved and non-authoritative
- Source task: one start preflight; Python watcher·repeated tool gate·removed source-guard CI `0건`
