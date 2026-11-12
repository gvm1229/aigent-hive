# 04. Plans, knowledge and closure

Canonical active plans: docs/plans/PLAN.md (goals, completion index, links/order; no checkboxes),
concern-owned fragments (unique criterion IDs), docs/state/CURRENT.md (evidence, blockers, unrun
checks, next work). Each stays below 8KiB. Archive chronology; backlog/archive do not count toward
completion. Count each acceptance assertion once.

Durable maintainer rules for repo agent behavior belong in `.agents/directives/`, or the owning plan for plan-scoped rules; never only in host-private memory (maintainer, 2026-11-12).

Save plans before execution unless explicitly waived and no other rule requires one. Update the
completion index at each transition; completion needs fresh evidence. Mark unsupported acceptance
deferred. Material plan changes update CURRENT and owning ADR. Implementers follow saved owning
steps/dependencies/evidence, not private planner reasoning.

| When | Required reference |
| --- | --- |
| Create/materially revise a plan | [Handoff contract](references/planning-contract.md) |
| Start/resume implementation | [Reconcile](references/plan-reconciliation.md) |
| Finish material source work; capture/restructure knowledge | [Knowledge/preservation](references/knowledge-and-preservation.md) |
| Close a bound Hive run | [Closure](references/run-closure.md) |
| Stable readiness | [Stable gate](references/stable-plan-gate.md) |

Load only the applicable procedure. Runtime scratch, chat and summaries never replace canonical state.
Continuation/status semantics: 01; release execution: 03; human document style: 08.

At closure reconcile request, criteria, worktree, evidence and authorized remote actions. Record
remaining work/owner/reason in the manifest and material handoffs in CURRENT. Completed subtasks
do not close unfinished parents. Without a Hive run binding, use this plan/session procedure;
never create consumer state here or claim verified runtime activation.
