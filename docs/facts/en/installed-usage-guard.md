---
schema_version: 1
pair_id: installed-usage-guard
topic_slug: installed-usage-guard
language: en
counterpart: ../ko/installed-usage-guard.md
title: "Installed Guard Target Boundary"
summary: "Per-target enforcement applies to Hive targets; the separately installed Windows monitor protects the Codex app across folders."
tags: [guard, source, usage]
aliases: ["Installed usage policy"]
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:c18be9c1e87bb1a34d5ffeffa5aab93b2d620b86723a4ccfc05e804103b16aa8"
  - "repo:docs/guides/installed-usage-guard.md#sha256:dc9e92ed1cb85336da05c2f030e20efcdebf15b60751b90777a2307ec8cefef4"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:eb62d9db75f946f948b4dada89fa141654e80135"
status: active
---

# Installed Guard Target Boundary

- Installed product owns usage enforcement; configured projects use max(global, project).
- Source uses global thresholds and user-root runtime; no source consumer state.
- Non-Hive folders receive no per-target enforcement, controls or runtime; setup-free Skills remain available.
- Session controls require an exact configured target. Unrelated damaged graph CURRENT is preserved and non-authoritative.
- Source tasks run one start preflight; no source-local Python watcher or repeated tool gate.
- Separately installed 0.12.1 Windows monitor protects the Codex app across folders, including non-Hive chats; it does not create consumer state in those folders.
