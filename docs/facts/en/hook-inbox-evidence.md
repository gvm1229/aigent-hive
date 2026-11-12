---
schema_version: 1
pair_id: hook-inbox-evidence
topic_slug: hook-inbox-evidence
language: en
counterpart: ../ko/hook-inbox-evidence.md
title: "Hook inbox evidence boundary"
summary: "Worker execution and interactive Claude feedback need separate evidence."
tags: [evidence, hooks, orchestration]
aliases: [hook-inbox-evidence]
sources:
  - "repo:docs/research/overhaul/OV150-001.md#sha256:3e262412a523bc8a34072d1a9e07839b2a261592ef6ff415151a7a50c3f8b501"
links: [orchestration-ownership]
reviewed_revision: "git:7a7bf6321c47e671c9b28f77c661b60f0302eeb2"
status: active
---

# Hook inbox evidence boundary

For OV150-001, a Windows host handler can consume a file envelope and call subscription-authenticated Codex. Synthetic hook input proves only the handler leg. Completion also needs an interactive Claude session that receives the official hook feedback and reports the matching result and receipt. Never replace this proof with headless Claude, a stored return JSON, or a worker exit code.
