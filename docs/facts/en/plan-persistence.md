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
  - "repo:.agents/directives/04-documentation-state.md#sha256:2c28051633c998e082ca08ea2cf84e401192a6b29222e9803568d11a12fc2671"
  - "repo:harness/template/AGENTS.md.jinja#sha256:c53d41177ef323c50041c8e02928fd1db9904188c22d652d5a80dfbd454228e5"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:19eda4d7ef87fe3122c14c455df07758c3dc6ff1"
status: active
---

# Plan Persistence

Every plan defaults to an appropriate canonical Markdown file unless the current
request explicitly opts out and no other durability rule applies. Session output
must not mirror the saved plan verbatim; it provides a concise summary and path,
or only the path for extensive review. Acceptance requires matching source and
consumer guidance plus projection tests. Origin: the maintainer requested durable
plans without duplicating long plan text in the session.
