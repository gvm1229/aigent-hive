# 01. Behavior Directive

Owns response behavior, work selection, prompt routing, continuation, and final task status.

## Communication

- Respond in Korean unless the maintainer explicitly requests another language for the current
  response. Message language alone does not change this preference.
- An authored or refined prompt defaults to English unless the maintainer requests another prompt
  language. Keep surrounding explanation in the selected response language.
- Use plain language and lead with the result, decision, or blocker. Introduce internal terms only
  after the concrete user-visible effect.
- Keep one base language per passage. In Korean, retain English only for proper names, commands,
  identifiers, paths, schema keys, exact UI labels, and terms without a clear Korean equivalent.
  Translate meaning rather than English word order; avoid mixed Korean-English compounds.
- In English, use ASD-STE100 Simplified Technical English: short direct sentences, concrete verbs,
  one main point, no idiom or vague pronouns.
- For each passed, failed, skipped, deferred, unverified, or unsupported result, name the scope,
  exact reason, actual host or platform execution, proven range, and unproven range.
- Human-readable project document style belongs only to `08-human-documentation-style.md`.

## Work selection

- Run one bounded Source Wiki lookup before knowledge-dependent source work. A source-root refusal
  from consumer retrieval is not a completed lookup.
- Resolve a source-development version before planning or implementation. An exact version named
  by the maintainer in the current request overrides the active plan. Otherwise bind the request
  to the product version and next numbered public test in `docs/plans/PLAN.md`. Do not move or
  suggest the work to a later version merely because a numbered test already exists; apply the
  post-test acceptance reset in `03-workflow.md` when product bytes change.
- Answer a simple question after that lookup without a plan, project edit, or unrelated Skill.
- Automatically load installed `aigent-hive:prompt-refine` in `refine-only` mode for explicit
  prompt authoring or material ambiguity. Before digest-bound approval, do not execute the refined
  prompt. Skip refinement for a sufficiently clear task, simple question, or explicit other Skill.
- Before implementation, identify outcome, constraints, ownership surfaces, verification, and stop
  condition. Prefer deletion or maintained existing capability over new infrastructure.
- Finish every safe in-scope action before presenting pending work. Ask only for a material choice,
  credential, irreversible action, external publication authority, or exact user-owned blocker.

## Continuation and closure

- `all todos`, `until completion`, `do not stop`, explicit implementation followed by `continue`,
  and equivalent terminal instructions keep the task active while any in-scope action is
  agent-owned. Agent-owned work includes inspection, fixes, tests, commits, permitted pushes, CI
  observation, qualification, and authorized publication.
- A progress report that identifies a remaining agent-owned action must not end the task. A failed
  test, stale reference, incomplete CI result, elapsed time, or partial host evidence is a next
  action, not a handoff.
- Before marking a whole Goal or task `blocked`, require a closure with no independent
  `agent-owned` criterion. Keep a partial blocker attached to its criterion and continue the rest.
- Abort continued work only when an exact blocker requires user manual action, Codex must be
  restarted, or every scoped criterion is complete.
- User cancel or interrupt takes priority and permits immediate stop.
- Before a final response, classify every remaining item as `agent-owned`,
  `awaiting-user-authority`, `awaiting-external-evidence`, or `blocked`. Continue when any
  `agent-owned` item remains. Use `blocked` only for a repeated run-wide condition with a recovery
  path; use `complete` only when scoped criteria and evidence are complete.
- A protected action excluded by the maintainer does not block other authorized work. Stable
  release authority and release mechanics belong to `03-workflow.md`.

## Evidence

- Separate verified facts from inference and use the smallest fresh check that proves the claim.
- Durable plan, state, and fact procedures belong to `04-documentation-state.md`; chat history and
  runtime scratch state never override those sources.
