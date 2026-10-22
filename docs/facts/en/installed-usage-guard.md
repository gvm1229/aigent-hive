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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:6a8b7909e224bde9bcf45ea91e8267219b6a7b1a4546f3b601705db94d144a88"
  - "repo:docs/guides/installed-usage-guard.md#sha256:0ead68f33595fcbd18d4d9c9ec38926645ca3cf22ffd895a02628c5e3c97a384"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:f6bec784d94c76ef79cda8ff28e60a0b05765805"
status: active
---

# Installed Guard Target Boundary

Configured projects use max(global, project); source uses global policy and user-root runtime, never source consumer state. Non-Hive folders receive no per-target controls or state, while setup-free Skills remain available. Session controls require an exact configured target; unrelated damaged graph CURRENT is not authority. Source tasks run one start preflight, not a source-local watcher or repeated tool gate. The qualified opt-in Windows monitor keeps Codex open and permits direct resume without a lock. Saved opt-in alone does not establish live monitoring; global monitor disable is allowed outside projects.
