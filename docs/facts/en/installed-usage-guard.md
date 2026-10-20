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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:836d4ac233d8e719a9cd1b5a09a1deee7111a7c4e208f95f07e277fa3c899b98"
  - "repo:docs/guides/installed-usage-guard.md#sha256:735fffd7960d1dca621b1de7bb2fa7f77ef232cda6c75c4165afda76c9f8da50"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:ccf59c00e326034d6d12ef9a9c8a52c91d5ff8cc"
status: active
---

# Installed Guard Target Boundary

Configured projects use max(global, project); source uses global policy and user-root runtime, never source consumer state. Non-Hive folders receive no per-target controls or state, while setup-free Skills remain available. Session controls require an exact configured target; unrelated damaged graph CURRENT is not authority. Source tasks run one start preflight, not a source-local watcher or repeated tool gate. The qualified opt-in Windows monitor keeps Codex open and permits direct resume without a lock. Saved opt-in alone does not establish live monitoring; global monitor disable is allowed outside projects.
