---
name: hive-editless-question
description: Answer repository-dependent read-only questions, explanations, audits, status reports, comparisons, and why/how-much inquiries about the Aigent Hive source without changing tracked files or external state. Use automatically when the user asks to inspect, explain, review, measure, compare, summarize, or report and does not explicitly request a mutation. Unlike aigent-hive:answer, this Skill may read repository files, Git history, tests, plans, and approved source Wiki data.
---

# Hive Editless Question

## Read-Only Contract

- Treat repository-dependent explanation, review, audit, comparison, inventory, history,
  progress, and status questions as read-only by default.
- Do not require the user to say `editless`, `read-only`, or `do not edit`.
- Inspect only the evidence needed to answer the question.
- Allow repository reads, Git history inspection, non-mutating validation commands, and
  approved source Wiki queries.
- Do not edit files, write artifacts, stage, commit, push, change persistent workflow
  state, or mutate external systems.
- Do not start implementation merely because the answer identifies missing work.

## Workflow

1. Run the source usage guard before any other repository action.
2. Identify the claims that require evidence.
3. Use the smallest relevant read-only checks.
4. Separate observed facts from estimates or recommendations.
5. Answer outcome-first with concrete paths, counts, revisions, or command evidence when
   they materially support the result.
6. Stop after the question is sufficiently answered.

## Boundaries

- Use `aigent-hive:answer` instead when the question is self-contained and requires no
  repository context or tools.
- If the user explicitly requests a bounded mutation in the same prompt, preserve the
  read-only contract for the question and perform only that named mutation through the
  applicable workflow.
- Ask before a destructive, irreversible, credential-gated, production, or materially
  scope-changing action.
- Do not load unrelated Skills, memory, orchestration, or subagents merely to answer a
  repository question.
- Use read-only external research only when repository evidence cannot establish a
  time-sensitive or externally defined fact.
