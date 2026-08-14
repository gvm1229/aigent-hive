---
name: update-summary
description: Draft a concise Korean subscriber update that compares the current Aigent Hive stable release with the previous stable release. Use only verified source-repository evidence. Use for release-update notes, subscriber announcements, or version-improvement summaries in this source workspace.
---

# Update Summary

Create subscriber-facing Korean release summaries for this source workspace only.

## Workflow

1. Identify the current public stable release and the preceding stable release from canonical release notes, facts, plans, and release metadata.
2. Extract only user-visible improvements that have verified evidence. Exclude future candidates, internal investigation notes, and unshipped changes.
3. Write a Korean Markdown title in this form: `# Aigent Hive v<current> 업데이트 내역:`.
4. Write one concise Korean bullet per improvement. State the observable improvement first. Retain exact identifiers only when they help subscribers act or distinguish a feature.
5. Compare only the two stable versions requested. Do not claim a future release, rebuild an artifact, publish a release, or change product harness files.

## Output rules

- Use Korean for the title and bullets unless the requester explicitly selects another output language.
- Match the concise subscriber format already used for version update notices.
- Preserve security boundaries. Describe blocked secrets or credentials only as a safety outcome; never include a secret-shaped example or value.
- State uncertainty when release evidence is missing or contradictory instead of inferring a claim.

## Scope boundary

This is an explicit source-project-only Skill. Keep it under `.agents/skills/update-summary/`. Do not add it to `harness/`, a product catalog, a release bundle, or a consumer project projection.
