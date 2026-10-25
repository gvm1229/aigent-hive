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
  - "repo:.agents/directives/01-behavior.md#sha256:54f993c4403e7de3612efa12f499bbf1c3a957ee5893875cb0c50204505b1f40"
  - "repo:AGENTS.md#sha256:8b785024ed26e2d916e39dc4b96c90c64171c8eb28f671c4c7f8af1de77d0a68"
  - "repo:docs/architecture/agent-directive-ownership.md#sha256:db0d5c52f75c5ce82354cde25673a5ff701bf99b3458f4f311b7844555b0df05"
  - "repo:harness/template/AGENTS.md.jinja#sha256:ab0902d90e75ac32219217b3617c46008f3a906a84a68453bf389fd69e9ec529"
  - "repo:scripts/check-agent-directives.py#sha256:d0520ff0a53524541cff7965258c89e2a2a2d03df1066312d854ac1ced41e5f9"
links: [agent-autonomous-continuation, artifact-boundaries, historical-project-base-coverage]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
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
