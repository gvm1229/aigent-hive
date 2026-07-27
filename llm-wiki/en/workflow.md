---
schema_version: 1
pair_id: workflow
topic_slug: workflow
language: en
counterpart: ../ko/workflow.md
title: "Source Development Workflow"
summary: "Surgical implementation, verification discipline, branch policy, and commit hygiene."
tags: [development, git, verification]
aliases: ["source development workflow"]
sources:
  - "repo:.agents/directives/00-editing-discipline.md#sha256:6ff1639897049dea7ccf710c88fe3bcb369d7edf7e62bcd62137ec70a7c7cc24"
  - "repo:docs/guides/branching-rules.md#sha256:c0b19cc2978f33002a980a7bf9fdb4563fcad8d5096781c3b9f15a0ba99a3304"
  - "repo:docs/guides/commit-rules.md#sha256:443986db38ba26db52106b49ef92d741b103f5b73f82d95e24f8bfcc20ed2887"
links: [crate-architecture, skill-routing, usage-hosts]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# Source Development Workflow

Before implementation, define the requested outcome, assumptions, ownership scope, verification,
and stop condition. Prefer the smallest change that satisfies the contract. Avoid speculative
abstractions, adjacent cleanup, and unrelated formatting. Every changed line should trace to a
requirement, defect, decision, or proof need.

Ordinary development occurs on `develop`. Stable integration reaches `main` through a pull request;
direct ordinary commits to `main`, branch deletion, and unapproved history rewriting are forbidden.
Before push, verify the remote and exact target ref.

Each commit owns one clear concern, stages only intended files, and uses a concise Korean
Conventional Commit title without automated co-author trailers. Completion requires fresh targeted
tests, wider lint or build checks proportional to risk, diff inspection, and explicit reporting of
any validation gap.
