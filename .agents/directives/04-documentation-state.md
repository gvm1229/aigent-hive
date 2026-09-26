# 04. Plans, knowledge and closure

One canonical active plan set: docs/plans/PLAN.md (goals, completion index, links/order; no checkboxes),
concern-owned active fragments (unique criterion IDs), docs/state/CURRENT.md (current evidence,
blockers, unrun checks, next work). Each stays below 8KiB. Archive chronology; backlog/archive do
not count toward completion. Never count the same acceptance assertion twice.

Persist plans before execution unless explicitly waived and no independent rule requires one.
Update the completion index at each transition; only fresh evidence permits completion. Preserve
unsupported acceptance as explicitly deferred. Material plan changes update CURRENT and owning ADR.
An implementer follows the saved owning step/dependencies/evidence, not the planner's private reasoning.

| When | Required reference |
| --- | --- |
| Create/materially revise a plan | [Handoff contract](references/planning-contract.md) |
| Start/resume implementation | [Reconcile](references/plan-reconciliation.md) |
| Finish material source work; capture/restructure knowledge | [Knowledge/preservation](references/knowledge-and-preservation.md) |
| Close a bound Hive run | [Closure](references/run-closure.md) |
| Stable readiness | [Stable gate](references/stable-plan-gate.md) |

Load only the applicable procedure. Runtime scratch, chat and summaries never replace canonical state.
Continuation/status semantics: 01; release execution: 03; human document style: 08.

At task closure reconcile request, owning criteria, worktree, evidence and authorized remote actions.
Record remaining work/owner/reason in the session manifest and a material handoff in CURRENT.
A completed subtask does not close its unfinished parent. Without a Hive run binding, use this
plan/session procedure; never create consumer state here or claim verified runtime activation.
