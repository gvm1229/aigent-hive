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
  - "repo:.agents/directives/03-workflow.md#sha256:5b882b23c1236ad4667243562add756432c100bfada68a970c4a6134a7bf73c1"
  - "repo:docs/guides/branching-rules.md#sha256:c0b19cc2978f33002a980a7bf9fdb4563fcad8d5096781c3b9f15a0ba99a3304"
  - "repo:docs/guides/commit-rules.md#sha256:9367805c05dc7f9f4f60dd95ea9fd7b7db22de2bd56060c5fbb9583f6ff6a925"
links: [crate-architecture, product-intent, skill-routing, usage-hosts]
reviewed_revision: "git:d46e9b7deb5c54fc7cec00c38483388ce563ff1d"
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

Each commit owns one independently reviewable and revertible concern. Wiki or documentation state,
product behavior, version metadata, and release activation are separate by default. A Wiki capture
and `hive --version` change require separate commits. Existing history is never rewritten merely to
apply the newer split policy.

Every completed source task receives its own commit before later completed work is combined with
it. Tasks that contain independently reviewable and revertible concerns are split further.
Sharing one request, session, milestone, or delivery window never permits unrelated completed
tasks in one commit.

Every plan that governs repository work is written into the tracked canonical plan set before
execution; chat or native plan state cannot be the sole authority. Completion requires fresh
targeted tests, proportional wider checks, diff inspection, and explicit validation gaps.
