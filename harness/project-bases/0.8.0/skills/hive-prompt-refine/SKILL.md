---
name: hive-prompt-refine
description: Create or refine a copy-ready prompt only when the user explicitly asks; never rewrite or execute ordinary questions or work requests.
---

# Hive Prompt Refine

Preserve the user's meaning while producing a concise, copy-ready prompt.

## Mode

- Default to `refine-only`.
- Use `refine-and-run` only when the user explicitly asks in the same request to execute the refined prompt.
- Do not rewrite ordinary questions or ordinary work requests.
- Do not treat urgency, autonomy language, or a request for a complete prompt as permission to run it.
- In `refine-only`, do not execute the prompt, read project files, call tools, write files, spawn subagents, create a run, or capture memory.

## Workflow

1. Preserve the original prompt as immutable input.
2. Extract the primary goal, intended audience or agent, required inputs, scope, constraints, prohibited actions, acceptance criteria, output shape, and stop conditions.
3. Identify only ambiguities that would materially change the result.
4. Ask at most one required ambiguity question at a time.
5. Leave unanswered nonessential details as explicit assumptions or placeholders. Do not invent facts.
6. Produce a provider-neutral prompt unless the user explicitly names a target host.
7. If a target host is explicit, add only the minimal host-specific syntax needed for that target.
8. Compare the refined prompt with the original before returning it. Preserve every must, must-not, scope boundary, target output, named tool or provider choice, and user authority boundary.
9. Avoid expanding a prompt that is already sufficiently precise.

If optional project grounding is explicitly requested, keep it a separate read-only action with its own capability decision. Do not silently read the project as part of refinement.

## Output

Return these sections:

1. `Intent summary`
2. `Assumptions and unresolved items`
3. `Refined prompt`

Structure the refined prompt only as far as useful:

- Goal
- Context and grounded inputs
- Required workflow
- Constraints and prohibited actions
- Acceptance and verification
- Output contract
- Stop, blocker, and escalation conditions

For explicit `refine-and-run`, show the refined prompt first, then hand execution to the normally resolved host or external orchestration owner. This Skill does not implement an execution loop.
