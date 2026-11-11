---
schema_version: 1
pair_id: harness-improvement-research
topic_slug: harness-improvement-research
language: en
counterpart: ../ko/harness-improvement-research.md
title: "Score-Based Hive Improvement Direction"
summary: "Hive combines controlled execution, task-specific scores, bounded harness trials, independent checks, and conditional reuse."
tags: [evaluation, orchestration, research]
aliases: ["Harness²", "score-based evaluation"]
sources:
  - "repo:docs/decisions/score-driven-improvement.md#sha256:8c19a8b5cba675d3789919303e76008546e81e5ddcabb94c0837072f92e6c344"
  - "repo:docs/plans/score-driven-improvement-0.14.0.md#sha256:a138c9ad92622a2460daa5046ecf71d8c486aef36939996c89fadab0356ee8e0"
links: [ai-learning-application-review, orchestration-ownership]
reviewed_revision: "git:7a7f314197b23f93f72e839bbb9f6226470a8f9d"
status: active
---

# Score-Based Hive Improvement Direction

The user withdrew Harness² as the ultimate goal. The final plan combines Hive execution control, task-specific scores, bounded harness trials, independent checks, and conditional experience reuse. Harness² is a reference only.

Plan: docs/plans/score-driven-improvement-0.14.0.md, twelve criteria. Artifact revision and harness improvement have separate state and one parent budget. Planning does not authorize implementation, installation, or release. Existing CLI work stays separate until execution-time integration.
