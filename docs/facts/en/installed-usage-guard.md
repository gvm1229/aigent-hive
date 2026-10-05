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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:b96d727dc4effe4c4ed77141927a7a67b4a5be047683544c4992b1e274b9ef55"
  - "repo:docs/guides/installed-usage-guard.md#sha256:c94975f1e11052ebf9c04e00066fe121229eea186945c056164d3a33c609df87"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:e22ff9036b04535dc9a9cc542f08b20143cfe40e"
status: active
---

# Installed Guard Target Boundary

- Installed product: sole usage-guard implementation
- Configured Hive project: `max(global, project)` and project-local session state
- Aigent Hive source: global threshold·user-root runtime·source `.hive/` files `0건`
- Non-Hive folder: enforcement·threshold mutation·session override·halt·runtime `0건`; setup-free Skills available
- Session control: explicit configured target only; unrelated malformed graph `CURRENT.md` preserved and non-authoritative
- Source task: one start preflight; Python watcher·repeated tool gate·removed source-guard CI `0건`
