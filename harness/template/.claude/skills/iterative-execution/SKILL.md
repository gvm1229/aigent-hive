---
name: iterative-execution
description: Execute a bounded persistent criterion loop through Hive orchestration events and host-native task receipts. Use for explicit iterative implementation whose terminal criteria require independent Judge evidence.
---

# Hive Iterative Execution

Run one bounded criterion loop while the active host retains task-launch ownership.

## Workflow

1. Require a Hive-configured project, exact run ID, finite criteria, retry and usage budgets,
   verified host capability, and an active signed authority. Stop on `unsupported` or `unverified`.
2. Run `$aigent-hive:usage-guard` before each dispatch. A limited result leaves the criterion
   pending and never authorizes success.
3. Commit `reserved` then `prepared` through `hive orchestration plan`. Give the resulting
   declarative envelope to the active host. Never launch a model or subagent process from Hive.
4. Commit only exact claim, launch, heartbeat, lookup, cancel, and final-result receipts. Ack loss
   becomes `dispatch-uncertain`; never reclaim without qualified non-launch proof.
5. Verify each criterion independently. Before its terminal acceptance, require a fresh reserved
   Judge result and external Ed25519 signature bound to the exact role, model, effort, definition,
   evidence, run head, and request digest.
6. Stop on repeated failure fingerprints, exhausted budget, cancellation, quarantine, or missing
   attestation. Recover only through `hive orchestration recover`.

## Role coverage

- Use this same bounded loop for planning after `$aigent-hive:ralph-loop`, independent review or
  QA after `$aigent-hive:package-review`, evidence-backed research after
  `$aigent-hive:research-best-practices`, and performance validation with declared measurements.
- Every role shares the same canonical event, receipt, evidence, usage, cancellation, and recovery
  path. Role labels never create a second scheduler or bypass the host-native envelope boundary.
- Terminal acceptance always requires the reserved independent Judge. `explicit` and `implicit`
  alter only the additional material-risk route; neither mode permits a terminal bypass.

## Boundaries

- Do not accept a criterion from an implementation agent's self-review.
- Do not invoke Judge for scheduler ticks, heartbeats, or retries.
- Do not call a provider API, handle provider credentials, spawn a process, or invoke OMX/OMC.
