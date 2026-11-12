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
  - "repo:docs/research/overhaul/OV150-001.md#sha256:7386967ec5249ecdb61460a251877f514a597a1fa6e997950f7112362844f8bb"
links: [orchestration-ownership]
reviewed_revision: "git:eb988a3637ec5ce3035de4cd758b133828d3c5c0"
status: active
---

# Hook inbox evidence boundary

For OV150-001, a Windows host handler can consume a file envelope and call subscription-authenticated Codex. Synthetic hook input proves only the handler leg. Completion also needs an interactive Claude session that receives the official hook feedback and reports the matching result and receipt. Never replace this proof with headless Claude, a stored return JSON, or a worker exit code.

UserPromptSubmit can fire without an assistant turn. Its queue files alone prove neither Stop nor worker dispatch. An output-free Stop must retain a recoverable pending request without claiming execution.
