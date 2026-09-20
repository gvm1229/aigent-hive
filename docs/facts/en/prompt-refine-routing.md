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
  - "repo:crates/hive-projection/src/lib.rs#sha256:94468852deebfdb0b71f20fdc1bafeaa0e20708282357ebd378f96ed12a84847"
  - "repo:harness/skills/prompt-refine/SKILL.md#sha256:bbd9a76fed57e1276aa94266709d79f656eafebc98e58723c5f8286b979399ed"
links: [orchestration-ownership, skill-routing]
reviewed_revision: "git:0340f8a26d14ebcc134c14e421d605f8022912f8"
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
