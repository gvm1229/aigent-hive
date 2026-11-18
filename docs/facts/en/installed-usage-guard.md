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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:1545e670dfd7be94e0d46f9e019d18c4489c2dfebb3c5e5ae7a3fd69d2b5b074"
  - "repo:docs/guides/installed-usage-guard.md#sha256:26f34a3f463b6cb2cf55da617e2c948406b73101dbe7d61e6a201e462f13a0cd"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Installed Guard Target Boundary

Configured projects use max(global, project); source uses global policy and user-root runtime, never source consumer state. Non-Hive folders receive no per-target controls or state, while setup-free Skills remain available. Session controls require an exact configured target; unrelated damaged graph CURRENT is not authority. Source tasks run one start preflight, not a source-local watcher or repeated tool gate. The qualified opt-in Windows monitor keeps Codex open and permits direct resume without a lock. Saved opt-in alone does not establish live monitoring; global monitor disable is allowed outside projects.
