---
schema_version: 1
pair_id: agent-directive-ownership
topic_slug: agent-directive-ownership
language: en
counterpart: ../ko/agent-directive-ownership.md
title: "Agent Directive Ownership"
summary: "Hive routes source and consumer work to one canonical directive per rule family and verifies size, route, projection, and duplicate-rule budgets."
tags: [directives, routing, v0-10]
aliases: ["Directive optimization"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:20905d49494df815461b4e9ffe6df89ee33ccb774510da2cfa10c98f0508b077"
  - "repo:AGENTS.md#sha256:d8fe84d5fe9bf291465651087a79135880c9b6f17e284e65a4eeb0891d851f2f"
  - "repo:docs/architecture/agent-directive-ownership.md#sha256:53476c7ca8f772d1d2bd956616d3b3f8235282a4c0643784e1a41895333cd2a9"
  - "repo:harness/template/AGENTS.md.jinja#sha256:7fdcd351b7d0624baa68d11bf9e850692c9eeaae13abb003295c8727f621543a"
  - "repo:scripts/check-agent-directives.py#sha256:4c9fe2ff89d0429b76c1e7a36fa2a3c5e9a953f29c592fde8b8199d793ab2332"
links: [agent-autonomous-continuation, artifact-boundaries, historical-project-base-coverage]
reviewed_revision: "git:64125db02505a9a696e870d23fa54feb125b8093"
status: active
---

# Agent Directive Ownership

The source `AGENTS.md` and consumer `AGENTS.md` projection are small routers. Detailed rule
families have one canonical directive owner, while generated entrypoints keep only approved
summaries. A static gate verifies byte budgets, route targets, current projection parity, and
non-allowlisted normalized rule duplicates. Historical project and user bases remain immutable.
Stable publication still requires explicit approval for the named version.
