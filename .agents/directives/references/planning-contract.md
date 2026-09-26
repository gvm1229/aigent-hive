# Implementation handoff contract

Read when creating or materially revising a plan. An implementer follows the saved plan;
they need not reload this authoring guide on each step.

The planner must resolve the design, not leave it for the implementer to rediscover. Assume
the next model has neither the planner's reasoning capacity nor its conversation history.
Do not hardcode model names or require a more capable model to finish an ordinary step.
Use detail proportional to risk: a small edit can use a short plan, but applicable fields
below must be concrete. Length and step count are not quality measures.

## Required decisions

- State the requested outcome, observable acceptance criteria, exclusions, active version,
  authority already granted, and actions needing separate approval.
- Inspect current code before naming files, symbols, commands, interfaces or dependencies.
  Label an unverified assumption; never invent an API or silently treat a guess as fact.
- Choose the approach and give its decisive reason. Record constraints and rejected alternatives
  only where their absence could cause a different implementation.
- Specify interfaces and data shapes, ownership and state transitions, compatibility, failure
  behavior, concurrency/atomicity when applicable, and what must remain unchanged.
- Identify the smallest files/symbols to change or create, reusable helpers, and prohibited edits.
  Links alone are not decisions: name the relevant section and what the implementer needs from it.

## Each implementation step

Use this compact record; omit fields only with a clear not-applicable reason:

1. **ID, dependencies and entry condition:** what must already be true; who owns this step.
2. **Read/change set:** exact paths and symbols; the narrow references needed now.
3. **Procedure:** ordered edits or pseudocode, input/output examples, branch conditions,
   error handling and preserved behavior. No unresolved choice hidden behind “implement robustly.”
4. **Verification:** exact command or reproducible procedure, expected observable result,
   negative cases, and what this evidence does not prove. Tests must check behavior, not just wording.
5. **Completion and recovery:** required artifact/evidence, checkpoint, rollback or recovery
   when needed, and a precise stop/escalation condition.

Finish with execution order, final integration checks, scoped completion criteria and the
handoff state. Store detailed step records in concern-owned fragments; keep the root index small.
Use existing plan IDs and avoid counting the same acceptance assertion twice.

## Unknowns and execution discipline

- Resolve material unknowns before handing off dependent implementation. If investigation is
  itself the next step, bound its question, allowed paths/actions, expected evidence and decision
  rule. Revise the plan after the result and before dependent edits; do not pass an open design choice
  to a cheaper implementer as an implementation task.
- The implementer first reconciles the plan with current files and completed evidence. A stale
  path, contradictory contract or failed assumption pauses only its dependent step.
- Follow the selected approach and scope. Do not replace architecture, expand scope, weaken a
  check or reinterpret approval to make progress. Report the discrepancy and evidence to the plan owner before dependent edits. Mechanical path
  corrections may preserve the selected contract; a material design change returns to the planner,
  not an unapproved worker redesign. Continue independent planned steps. Ask the user only for
  a new material choice or authority, not for already authorized implementation.
- Record meaningful changes once in the owning plan. Do not copy the full plan into chat or
  repeat it before every tool call.

## Handoff review

Before calling a plan ready, mentally walk through its first implementation step as a worker with only the saved plan.
Can that worker identify the next file/symbol, required change, success signal and failure path
without inventing a design decision? Repeat for cross-step dependencies; this does not require an extra model call or subagent. Fix gaps before handoff;
never mark implementation complete merely because the plan is detailed.
