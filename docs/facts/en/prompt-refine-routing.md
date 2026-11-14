---
schema_version: 1
pair_id: prompt-refine-routing
topic_slug: prompt-refine-routing
language: en
counterpart: ../ko/prompt-refine-routing.md
title: "Prompt Refine Approval Routing"
summary: "Explicit prompt authoring requires execution approval; ambiguous work retains its authorized route."
tags: [prompt, routing, skill]
aliases: ["Prompt approval gate"]
sources:
  - "repo:crates/hive-projection/src/lib.rs#sha256:4d4d64b5b580a455b365a1039de568b39ff395072dc0d653e8ea495238a73fe4"
  - "repo:harness/skills/prompt-refine/SKILL.md#sha256:bbd9a76fed57e1276aa94266709d79f656eafebc98e58723c5f8286b979399ed"
links: [orchestration-ownership, skill-routing]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Prompt Refine Approval Routing

Explicit prompt authoring uses `refine-only`. Delivering the prompt completes that writing task;
`awaiting-approval` refers only to later execution. A local hash of the authored text is allowed,
but project inspection, writes, memory capture, and task execution remain outside refinement.
Explicit same-request run intent or later exact approval authorizes execution.

Ambiguous ordinary work keeps `RunWork` and the host-native route with an optional refinement
suggestion, not automatic Skill activation. Continue authorized investigation and ask only for
material user choices or new authority. The 0.10.2 instruction review replaces the earlier
automatic ambiguity-to-refinement policy; no prompt-classifier hook is introduced.
