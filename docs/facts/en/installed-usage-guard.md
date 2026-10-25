---
schema_version: 1
pair_id: installed-usage-guard
topic_slug: installed-usage-guard
language: en
counterpart: ../ko/installed-usage-guard.md
title: "Installed Guard Target Boundary"
summary: "Configured targets and opt-in app-preserving interruption remain separate from monitor health."
tags: [guard, source, usage]
aliases: ["Installed usage policy"]
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:9b858bef8831bed659b0f977ecd9ba84111800176d9dfc048b1d46c4c6ab8115"
  - "repo:docs/guides/installed-usage-guard.md#sha256:0ead68f33595fcbd18d4d9c9ec38926645ca3cf22ffd895a02628c5e3c97a384"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Installed Guard Target Boundary

Configured projects use max(global, project); source uses global policy and user-root runtime, never source consumer state. Non-Hive folders receive no per-target controls or state, while setup-free Skills remain available. Session controls require an exact configured target; unrelated damaged graph CURRENT is not authority. Source tasks run one start preflight, not a source-local watcher or repeated tool gate. The qualified opt-in Windows monitor keeps Codex open and permits direct resume without a lock. Saved opt-in alone does not establish live monitoring; global monitor disable is allowed outside projects.
