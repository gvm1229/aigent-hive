---
name: update-summary
description: Draft a concise Korean subscriber update that compares the current Aigent Hive stable release with the previous stable release. Use only verified source-repository evidence. Use for release-update notes, subscriber announcements, or version-improvement summaries in this source workspace.
---

# Update Summary

Create subscriber-facing Korean release summaries for this source workspace only.

## Workflow

1. Identify the current public stable release and the preceding stable release from canonical release notes, facts, plans, and release metadata.
2. Extract only verified improvements that an end user can notice, use, avoid, or understand differently. Exclude future candidates, internal investigation notes, unshipped changes, and developer- or contributor-only work.
3. Write a Korean Markdown title in this form: `# Aigent Hive v<current> 업데이트 내역:`.
4. Write one concise Korean bullet per improvement. State the observable improvement first. Retain exact identifiers only when they help subscribers act or distinguish a feature.
5. Compare only the two stable versions requested. Do not claim a future release, rebuild an artifact, publish a release, or change product harness files.

## End-user relevance

- Include a change only when it changes the installed product, installation or update experience, user workflow, safety protection, or user-facing understanding of a usable feature.
- Exclude GitHub Release language order or formatting, CI and test procedures, release verification records, repository plans, source documentation, internal catalog or projection work, and contributor workflows unless the change directly alters a subscriber action or outcome.
- In particular, do not report an English-first and Korean-second GitHub Release description format. It is a publication detail for developers and contributors, not a subscriber improvement.
- Test every candidate bullet with this question: “What can the subscriber now do, avoid, or understand differently?” Omit it when there is no direct answer.

## Output rules

- Use Korean for the title and bullets unless the requester explicitly selects another output language.
- Match the concise subscriber format already used for version update notices.
- Every bullet must state the subscriber benefit or changed capability; do not expose implementation, process, or publication details as an improvement.
- Preserve security boundaries. Describe blocked secrets or credentials only as a safety outcome; never include a secret-shaped example or value.
- State uncertainty when release evidence is missing or contradictory instead of inferring a claim.

## Scope boundary

This is an explicit source-project-only Skill. Keep it under `.agents/skills/update-summary/`. Do not add it to `harness/`, a product catalog, a release bundle, or a consumer project projection.
