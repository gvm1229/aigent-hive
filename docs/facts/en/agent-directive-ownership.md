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
  - "repo:.agents/directives/01-behavior.md#sha256:49c79d8137a1e1d3cdcb06b60fb7e2708e2cf0ae04018a6c9c866ecb0a65a333"
  - "repo:AGENTS.md#sha256:8b785024ed26e2d916e39dc4b96c90c64171c8eb28f671c4c7f8af1de77d0a68"
  - "repo:docs/architecture/agent-directive-ownership.md#sha256:db0d5c52f75c5ce82354cde25673a5ff701bf99b3458f4f311b7844555b0df05"
  - "repo:harness/template/AGENTS.md.jinja#sha256:02cfaea5d05fc9e51b5a368cc70290f6f2b041912433bf640ad2ccaa0f6b2c39"
  - "repo:scripts/check-agent-directives.py#sha256:d0520ff0a53524541cff7965258c89e2a2a2d03df1066312d854ac1ced41e5f9"
links: [agent-autonomous-continuation, artifact-boundaries, historical-project-base-coverage]
reviewed_revision: "git:c06e1d0e8077c4fb0a20bb209c2580570454f354"
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
