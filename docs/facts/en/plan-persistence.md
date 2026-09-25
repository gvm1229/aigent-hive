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
  - "repo:.agents/directives/04-documentation-state.md#sha256:8b4ab599048ee698e44869ef8c2d270c45e527873238993f2964ea0aee65456b"
  - "repo:.agents/directives/references/planning-contract.md#sha256:4136a403ed93af42bed844f30e7ed48309ab9ed535c3deb91b1d264dab2fda33"
  - "repo:docs/plans/README.md#sha256:85944730779c8686d4f436fe735f8e65b0ee34f8e5dee048103a8e85cd3f508a"
  - "repo:harness/template/AGENTS.md.jinja#sha256:02cfaea5d05fc9e51b5a368cc70290f6f2b041912433bf640ad2ccaa0f6b2c39"
links: [automated-user-handoff, source-development]
reviewed_revision: "git:64125db02505a9a696e870d23fa54feb125b8093"
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
