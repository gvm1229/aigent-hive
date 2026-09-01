---
schema_version: 1
pair_id: agent-directive-ownership
topic_slug: agent-directive-ownership
language: en
counterpart: ../ko/agent-directive-ownership.md
title: "Agent Directive Ownership"
summary: "Hive routes each rule family to one canonical directive, keeps unspecified development on the active version, and verifies size, route, projection, and duplicate-rule budgets."
tags: [directives, routing, v0-10]
aliases: ["Directive optimization"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:4b22be47789033b39654596bb345fd56017e54bf4cd8ef12ad1cac7ae9c8e4d4"
  - "repo:AGENTS.md#sha256:d1a4541174db15faf38f3c90432fbea8cb4b4da6448bfccce2a7e069982031b6"
  - "repo:docs/architecture/agent-directive-ownership.md#sha256:77cf75a4f3c815fa9a75635e7b43f95035bdeebf0ca47a1bd7c94df14c7942c4"
  - "repo:harness/template/AGENTS.md.jinja#sha256:7fdcd351b7d0624baa68d11bf9e850692c9eeaae13abb003295c8727f621543a"
  - "repo:scripts/check-agent-directives.py#sha256:4c9fe2ff89d0429b76c1e7a36fa2a3c5e9a953f29c592fde8b8199d793ab2332"
links: [agent-autonomous-continuation, artifact-boundaries, historical-project-base-coverage]
reviewed_revision: "git:f34c524da540a97d6c2810fb1d0b092bbf1421ed"
status: active
---

# Agent Directive Ownership

The source `AGENTS.md` and consumer `AGENTS.md` projection are small routers. Detailed rule
families have one canonical directive owner, while generated entrypoints keep only approved
summaries. A static gate verifies byte budgets, route targets, current projection parity, and
non-allowlisted normalized rule duplicates. Historical project and user bases remain immutable.
Stable publication still requires explicit approval for the named version.
Without a newly named version, source development stays on the product version and next numbered
public test in the active plan; the agent must not invent a later destination.
