---
name: hive-simple-question
description: Answer a self-contained simple question without project context, tools, memory, extra Skills, subagents, orchestration, or persistent run state. Use only for general questions that can be answered from the user's message and ordinary reasoning; do not use for repository-dependent, current external-data, mutation, or multi-step work.
---

# Hive Simple Question

Answer directly from the current user message and ordinary reasoning.

## Isolation

- Do not read project files, `.hive/`, project memory, Wiki, run state, or role state.
- Do not load another Skill, call a tool, access the network, spawn a subagent, or invoke OMX, OMC, or another orchestration layer.
- Do not create or modify files, capture the exchange as memory, or create a persistent run.
- Keep the answer proportionate to the question.

## Boundary

If an accurate answer depends on repository state, project memory, current external data, mutation, or multi-step execution:

1. Do not inspect, transition, or write automatically.
2. State briefly that the request is outside the isolated simple-question path.
3. Suggest the appropriate explicit action, such as `RunWork` or `QueryKnowledge`.
