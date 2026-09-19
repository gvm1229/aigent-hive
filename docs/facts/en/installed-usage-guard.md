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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:267309f9d7f56095b6cc00f4602aeaf1249e0e8988f7fc5bd897dff04e2be5f5"
  - "repo:docs/guides/installed-usage-guard.md#sha256:9cf01b711909bca15472e95b5a25325b7094df4115795b6fdf04cf2bc3f017f5"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:4afd5ba4483d98f63ae42c065837f9b010506657"
status: active
---

# Installed Guard Target Boundary

- Installed product: sole usage-guard implementation
- Configured Hive project: `max(global, project)` and project-local session state
- Aigent Hive source: global threshold·user-root runtime·source `.hive/` files `0건`
- Non-Hive folder: enforcement·threshold mutation·session override·halt·runtime `0건`; setup-free Skills available
- Session control: explicit configured target only; unrelated malformed graph `CURRENT.md` preserved and non-authoritative
- Source task: one start preflight; Python watcher·repeated tool gate·removed source-guard CI `0건`
