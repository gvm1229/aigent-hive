# 01. Behavior Directive

This directive governs agent behavior while developing Aigent Hive.

## Communication

- Respond to the maintainer in Korean unless explicitly asked to use another language.
- Lead with the result, decision, or blocker.
- Keep progress updates concise and evidence-based.
- Ask only when missing information would materially change the product, create irreversible risk, or require credentials or external publication authority.

## Work Selection

- Answer simple questions directly without loading unrelated project memory or starting a planning workflow.
- Route explicit prompt authoring or refinement intent to the source `hive-prompt-refine` Skill in `refine-only` mode unless the same request explicitly authorizes execution.
- For an ordinary work prompt whose goal, scope, constraints, acceptance criteria, or output contract is materially ambiguous or missing, add one concise optional refinement suggestion while continuing every safe, discoverable part of the task.
- A refinement suggestion must not rewrite the prompt, load `hive-prompt-refine`, authorize execution, or interrupt a sufficiently clear ordinary task or simple question.
- For implementation, identify the requested outcome, constraints, touched ownership surfaces, verification, and stop condition before editing.
- Prefer deletion, an existing dependency, or an existing host capability over a new abstraction.
- Do not copy external project rules unless they are explicitly selected and project-neutral.
- Keep changes surgical: every touched artifact must map to a requirement, defect, decision, or verification need.

## Evidence

- Separate verified facts from inference.
- Inspect current files and upstream documentation before making version-sensitive claims.
- Run the smallest fresh check that can prove each completion claim.
- Record durable decisions in `docs/`; do not rely on chat history, `.omx/`, transcripts, or compaction summaries as project memory.
