# Project harness behavior

Owns material-work planning, continuation, closure, host execution, and stable publication authority.
Knowledge, upgrade, and concurrent-edit procedures belong to the numbered sibling directives.

## Work and planning

- English responses use ASD-STE100 Simplified Technical English. In Korean, Translate meaning rather than English word order.
- In Korean, do not write `benign한 source claim ID`; use `원본 지식 항목 식별자` and avoid replaceable mixed-language compounds.
- Before producing Korean, load `04-korean-language.md` and apply its draft, local inspection,
  bounded rewrite, verification, and exact-draft fallback contract.
- Treat the repository as an independent consumer project and `.hive/config/` plus canonical
  Markdown as authority.
- Write a material plan to project Markdown before execution unless the user opts out. Do not copy
  the complete saved plan into chat.
- Finish every safe in-scope agent-owned action before a handoff. Report only exact user-owned or
  external evidence requirements.
- Preserve user and third-party bytes outside Hive-owned paths and marker blocks.

## Continuation and closure

- `all todos`, `until completion`, `do not stop`, explicit implementation followed by
  `continue`, and equivalent requests remain active while any scoped action is agent-owned.
- Classify remaining work as `agent-owned`, `awaiting-user-authority`,
  `awaiting-external-evidence`, or `blocked`. A progress report is not closure.
- Before marking a whole Goal or task `blocked`, require no independent agent-owned criterion.
  Keep a partial host, fixture, test, stale-reference, or evidence failure attached to its criterion
  and continue the rest.
- Abort continued work only for an exact user-owned manual blocker, a required Codex restart, or
  completed criteria. User cancel or interrupt takes priority and permits immediate stop.
- Hive hooks may return only their bounded approved result. They never mutate the host Goal, task,
  or canonical run state.
- For a bound run, call `hive run closure --target <project-root> --run <run-id> --output json`
  before a completion claim. Inspect `data.closure.ready_for_final` and remaining criteria, not
  merely exit zero or the command code. Match the run, revision, and current evidence to the whole
  requested task; a successful subtask or disposable test cannot close its parent. Cancellation
  permits a stop, not a success claim. Reconcile blocked results with the policy above.
- Without a bound run, reconcile the project plan and remaining actions directly; do not claim
  verified execution. A pending required CI check calls for bounded waiting or independent work,
  not a final progress-only handoff. Reading these rules does not install a hook or prove that
  the host intercepts final responses.

## Release authority

- Every release request defaults to implementation, verification, or a uniquely numbered public
  test.
- Stable tag, protected-branch integration, publication, and installation require the user's
  current, version-specific approval. Never infer it from `release`, `ship`, `continue`,
  `all todos`, successful tests, or a ready report.

## Host and Skill boundary

- Start a new run with verified host-native capabilities. Use OMX or OMC only after explicit user
  selection and preserve an existing pinned owner, including a 0.8.x external owner.
- The host owns model, subagent, and Judge launches. Hive prepares declarative envelopes and typed
  receipts; `spawned=false` is mandatory.
- Missing or unverified capability returns truthful unsupported or `dispatch-uncertain` without a
  provider call, process launch, automatic backend switch, watcher, or fallback hook.
- Activate an optional Skill or hook only after exact scope and digest approval.
