# 01. Behavior Directive

This directive governs agent behavior while developing Aigent Hive.

## Communication

- Respond to the maintainer in Korean unless explicitly asked to use another language.
- Keep the selected response language consistent throughout each answer. When writing in Korean,
  prefer Korean vocabulary and syntax. Keep English only for proper nouns, product or package
  names, commands, code identifiers, paths, schema keys, exact UI labels, and terms without a
  clear Korean equivalent. Do not insert replaceable English general nouns into Korean prose.
- When writing in English, write the full passage in English except for exact Korean names,
  literals, quotations, or text the user explicitly asks to preserve.
- Lead with the result, decision, or blocker.
- Keep progress updates concise and evidence-based.
- Never gain brevity by removing a qualifier needed to interpret a result. For every passed,
  failed, skipped, deferred, unverified, or unsupported item, name the affected scope, exact
  reason, relationship to the current host or platform, whether it actually ran, and what the
  result does and does not prove. Do not use a platform adjective such as "Windows-only" or
  "Unix-only" without stating whether the current platform ran or skipped that item and why.
- Ask only when missing information would materially change the product, create irreversible risk, or require credentials or external publication authority.

## Work Selection

- Answer simple questions directly without loading unrelated project memory or starting a planning workflow.
- Route explicit prompt authoring or refinement intent to the source `hive-prompt-refine` Skill in `refine-only` mode unless the same request explicitly authorizes execution.
- For an ordinary work prompt whose goal, scope, constraints, acceptance criteria, or output contract is materially ambiguous or missing, add one concise optional refinement suggestion while continuing every safe, discoverable part of the task.
- A refinement suggestion must not rewrite the prompt, load `hive-prompt-refine`, authorize execution, or interrupt a sufficiently clear ordinary task or simple question.
- For implementation, identify the requested outcome, constraints, touched ownership surfaces, verification, and stop condition before editing.
- Before presenting a to-do list, pending action plan, blocker list, or user handoff, complete
  every safe, in-scope, automatable action that does not require new user authority,
  credentials, a protected external mutation, or a materially different product decision.
- Do not shift an automatable action to the user because it is later in a plan or more convenient
  to describe than execute.
- After that automation, present only genuinely user-owned actions as a concise ordered guide.
  Give each action's exact location, command or operation, expected result or return evidence, and
  the reason user authority is required. List failed or impossible work separately with its cause
  and recovery path.
- Prefer deletion, an existing dependency, or an existing host capability over a new abstraction.
- Do not copy external project rules unless they are explicitly selected and project-neutral.
- Keep changes surgical: every touched artifact must map to a requirement, defect, decision, or verification need.

## Evidence

- Separate verified facts from inference.
- Inspect current files and upstream documentation before making version-sensitive claims.
- Run the smallest fresh check that can prove each completion claim.
- Record durable decisions in `docs/`; do not rely on chat history, `.omx/`, transcripts, or compaction summaries as project memory.
