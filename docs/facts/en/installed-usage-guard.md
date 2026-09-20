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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:b2d3c7a9a42ce53e2ab8806401efb6e7076d7550843f0a09dd7158de56eee08f"
  - "repo:docs/guides/installed-usage-guard.md#sha256:c94975f1e11052ebf9c04e00066fe121229eea186945c056164d3a33c609df87"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:72bd93df896956e10c30013db3bc9232b7a4a3ca"
status: active
---

# Installed Guard Target Boundary

- Installed product: sole usage-guard implementation
- Configured Hive project: `max(global, project)` and project-local session state
- Aigent Hive source: global threshold·user-root runtime·source `.hive/` files `0건`
- Non-Hive folder: enforcement·threshold mutation·session override·halt·runtime `0건`; setup-free Skills available
- Session control: explicit configured target only; unrelated malformed graph `CURRENT.md` preserved and non-authoritative
- Source task: one start preflight; Python watcher·repeated tool gate·removed source-guard CI `0건`
