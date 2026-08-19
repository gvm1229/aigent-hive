---
schema_version: 1
pair_id: plan-persistence
topic_slug: plan-persistence
language: en
counterpart: ../ko/plan-persistence.md
title: "Plan Persistence"
summary: "Plans default to canonical Markdown and session references stay concise."
tags: [documentation, plan, state]
aliases: ["Markdown plan authority"]
sources:
  - "repo:.agents/directives/04-documentation-state.md#sha256:e941e74431e44442bb5940df43832b72ecfdcc4f3cb4963462ce6ee5ada2a32f"
  - "repo:docs/plans/README.md#sha256:85944730779c8686d4f436fe735f8e65b0ee34f8e5dee048103a8e85cd3f508a"
  - "repo:harness/template/AGENTS.md.jinja#sha256:33c0da7ba5156ea1aa0ccc08a8e4f88343cf5f6f896994a7d8b830ac0ad6bb74"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:a833ba22d805fedce23cd74aa326b574b77280f4"
status: active
---

# Plan Persistence

Every plan defaults to an appropriate canonical Markdown file unless the current
request explicitly opts out and no other durability rule applies. Session output
must not mirror the saved plan verbatim; it provides a concise summary and path,
or only the path for extensive review. Acceptance requires matching source and
consumer guidance plus projection tests. Origin: the maintainer requested durable
plans without duplicating long plan text in the session. `PLAN.md` revision is a
monotonic integer change counter: historical `1.99` followed by `2.00` means 99 then
100, not a new plan generation.
