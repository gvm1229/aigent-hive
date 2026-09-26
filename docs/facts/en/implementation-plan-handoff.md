---
schema_version: 1
pair_id: implementation-plan-handoff
topic_slug: implementation-plan-handoff
language: en
counterpart: ../ko/implementation-plan-handoff.md
title: "Implementation Plan Handoff"
summary: "The planner resolves material design choices before another model implements the saved steps."
tags: [directives, planning]
aliases: []
sources:
  - "repo:.agents/directives/references/planning-contract.md#sha256:4136a403ed93af42bed844f30e7ed48309ab9ed535c3deb91b1d264dab2fda33"
links: [agent-directive-ownership, plan-persistence]
reviewed_revision: "git:453bda5786f55a11c6e2c03a88bcb13bce5171ac"
status: active
---

# Implementation Plan Handoff

A Hive source plan must be usable by a different implementer without the planner's conversation or reasoning capacity. It specifies paths/symbols, selected design, ordered changes, data/error/compatibility rules, verification and recovery. Material design discrepancies return to the plan owner before dependent edits; independent planned work continues. Implementers read the owning step, not all planning references. This is a source-development rule, not a change to installed consumer guidance.
