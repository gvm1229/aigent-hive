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
  - "repo:docs/research/overhaul/OV150-001.md#sha256:e4aca1efb604fa5ca8f2a993142f86c7be14add5395c9db1f2a2634ba9fbd46a"
links: [orchestration-ownership]
reviewed_revision: "git:7a7bf6321c47e671c9b28f77c661b60f0302eeb2"
status: active
---

# Hook inbox evidence boundary

For OV150-001, a Windows host handler can consume a file envelope and call subscription-authenticated Codex. Synthetic hook input proves only the handler leg. Completion also needs an interactive Claude session that receives the official hook feedback and reports the matching result and receipt. Never replace this proof with headless Claude, a stored return JSON, or a worker exit code.
