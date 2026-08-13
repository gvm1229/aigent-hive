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
  - "repo:.agents/directives/04-documentation-state.md#sha256:44913afc655f527245720594f16a92c87061abfc28280f1a2834ad328b336be5"
  - "repo:docs/plans/PLAN.md#sha256:c005e38b9fada1a95aacf2ae7b443cacaf7f192ac4b72bdb7f506427f48bb621"
  - "repo:docs/plans/README.md#sha256:7fca19e770b1b99b647a893517b50bcf6e6eb136e3c84ae52ba1258267087df0"
  - "repo:harness/template/AGENTS.md.jinja#sha256:9b45162b48b5d6eedcce0d0b6bc2d63613776fc1089a7d2878941897d0d506c5"
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
100, not a new plan generation. Current normalized value: `191`.
