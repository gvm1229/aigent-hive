---
name: verified-workflow
description: (verified-workflow) Build and execute a bounded evidence-gated Hive workflow with dependency edges, host-native receipts, retry limits, independent verification, steering, and recovery. Use for complex work that requires more than ordinary continuation.
---

# Hive Verified Workflow

Use only when the active run needs dependency edges, intermediate evidence, bounded retry,
independent verification, steering, or exact recovery. The active host owns every model, task, and
Judge launch. Hive owns only canonical state, prepared envelopes, receipts, and verification.

## Workflow

1. Require an exact target, run ID, finite criteria, retry and usage budgets, verifier roles, and
   fresh host capability evidence. If a required capability is `unsupported` or `unverified`,
   return `host_capability_unsupported` without a dispatch request or fallback.
2. Define the dependency graph with criterion ownership, evidence predicates, retry policy, and
   independent verifier for every node. Initialize and validate it:

   ```text
   hive loop initialize --target <project-root> --graph <graph.md> --output json
   hive loop validate --target <project-root> --run <run-id> --output json
   ```

   Stop on a cycle, self-edge, unreachable node, orphan criterion, missing evidence predicate, or
   invalid terminal transition.
3. Before every automatic dispatch, run `$aigent-hive:usage-guard`. A limited or unknown result
   leaves the node pending and never authorizes success.
4. Commit `reserved` then `prepared` through `hive orchestration plan`, or prepare one validated
   loop node through `hive loop prepare`. Bind the exact graph revision, capability evidence,
   usage authorization, role, node, attempt, and brief digest. Accept only
   `prepared_only=true` and `spawned=false`.
5. Give that envelope to the active host. Hive never launches a model, subagent, Judge, or process.
   Record only exact claim, launch, heartbeat, lookup, cancellation, and result receipts. Lost
   acknowledgement is `dispatch-uncertain`; never reclaim without qualified non-launch proof.
6. Verify each criterion independently. Use a deterministic verifier by default. A Judge node or
   elevated risk uses `$aigent-hive:adversarial-judge` and the authenticated quorum contract; an
   implementation agent never accepts its own result.
7. Retry only inside the node's declared budget. Stop on repeated failure fingerprints, exhausted
   budget, cancellation, quarantine, or missing attestation.
8. Before a whole Goal or task becomes `blocked`, run closure against the current criterion set.
   A partial host, fixture, or external-evidence failure remains attached to that criterion while
   every independent agent-owned criterion continues. A hook may return one bounded nudge and
   never mutates the host Goal or task.
9. Record topology changes only through `hive loop steer` with the reason, affected edges, user
   boundary, and new immutable revision. Recover a new session only through `hive loop recover` or
   `hive orchestration recover`.

## Natural continuation routing

Natural continuation selects this workflow only when at least two of the following signals are
present: three-step dependency, intermediate evidence gate, bounded retry, independent verifier,
topology steering, or exact recovery. Task length and a bare `continue` request do not select it.

User override:

- `간단한 continuation`: normal Goal·closure path
- `검증형 workflow`: this Skill
- `retry 없음`: no retry policy

## Boundaries

- Do not call a provider API, access credentials, spawn a process, or invoke OMX/OMC.
- Do not create a second scheduler, mutate a host Goal, or infer host capability.
- Do not use a Judge for scheduler ticks, heartbeats, ordinary retries, or an implementation
  agent's self-review.
- Do not edit canonical run files directly. Use the signed Hive CLI and immutable revisions.
