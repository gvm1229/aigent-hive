---
schema_version: 1
pair_id: installed-usage-guard
topic_slug: installed-usage-guard
language: en
counterpart: ../ko/installed-usage-guard.md
title: "Installed Guard Target Boundary"
summary: "Per-target checks remain active; qualified Windows monitoring is locally installed."
tags: [guard, source, usage]
aliases: ["Installed usage policy"]
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:fb83162490270dc48650dd60c0a93c321c8811915d863fd102e478f9d22967f4"
  - "repo:docs/guides/installed-usage-guard.md#sha256:369a6681fcc7415c711ab7d5b9d1dae879b9d6fc188261af10a4ffcb5327a6f2"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:da2797509ea7e0ef94783877e3c2cb69c6bfd490"
status: active
---

# Installed Guard Target Boundary

- Installed product owns usage enforcement; configured projects use max(global, project).
- Source uses global thresholds and user-root runtime; no source consumer state.
- Non-Hive folders receive no per-target enforcement, controls or runtime; setup-free Skills remain available.
- Session controls require an exact configured target. Unrelated damaged graph CURRENT is preserved and non-authoritative.
- Source tasks run one start preflight; no source-local Python watcher or repeated tool gate.
- The 0.12.1 Windows monitor is locally installed and enabled on Codex 26.1013.4894.0. It stops runs once without a resume lock; unrelated folders receive no consumer state.
