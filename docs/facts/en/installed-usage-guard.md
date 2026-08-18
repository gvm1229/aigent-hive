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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:6c5febe7ae1ac1a892f7ac412c40d1b8d9ae339fe73fa8153faf9bb22051e1c0"
  - "repo:docs/guides/installed-usage-guard.md#sha256:3224f7e04c9025cd788e14506295a723f1d87c97d59f9e629dcfe9bddcb1a302"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:39569b7a2a7c67f8ab19010db8c4df32da470f86"
status: active
---

# Installed Guard Target Boundary

- Installed product: sole usage-guard implementation
- Configured Hive project: `max(global, project)` and project-local session state
- Aigent Hive source: global threshold·user-root runtime·source `.hive/` files `0건`
- Non-Hive folder: enforcement·threshold mutation·session override·halt·runtime `0건`; setup-free Skills available
- Session control: explicit configured target only; unrelated malformed graph `CURRENT.md` preserved and non-authoritative
- Source task: one start preflight; Python watcher·repeated tool gate·removed source-guard CI `0건`
