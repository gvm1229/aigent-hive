---
name: update-summary
description: Write evidence-based Korean release announcements that promote Aigent Hive features and give subscribers reasons to update. Compare the requested releases; support clearly marked prerelease review drafts. Source-project-only.
---

# Update Summary

Create subscriber-facing Korean release summaries for this source workspace only. These are product
advertisements: make users want to update by showing what is new and why it matters. Ground every
claim in verified evidence; persuasion never permits invented capabilities, hidden costs, or guarantees.

## Workflow

1. Identify the requested baseline and target releases from canonical release notes, facts, plans, and release metadata. Normally compare consecutive stable releases; for an explicitly requested unreleased target, compare it with the latest stable and label the result a review draft.
2. Extract only verified improvements that an end user can notice, use, avoid, or understand differently. Exclude future candidates, internal investigation notes, unshipped changes, and developer- or contributor-only work.
3. Write a Korean Markdown title in this form: `# Aigent Hive v<current> 업데이트 내역:`.
4. Write one concise Korean main bullet per change, with practical examples and relevant limits in a nested list. Lead with the feature addition or improvement and its benefit; place setup choices and costs below it.
5. Compare only the baseline and target identified above. Do not describe a review draft as a released stable version, rebuild an artifact, publish a release, or change product harness files.
6. When preparing a stable release, save the exact Korean title and bullets to `docs/releases/<current>.subscriber.ko.md`. This is the canonical Discord message payload for that stable release.
7. Before stable approval, a clearly marked review draft may use the same path for the active product version. It must say that it is not published or sent, and it must not describe unverified work as released.

## Feature positioning

- Classify each change against the baseline as new, improved, fixed, or renamed. Say `추가` or `도입` for a new capability; do not call an existing capability new or make a new feature sound like routine settings guidance.
- Keep meaningful core technical names such as `벡터 데이터베이스` and `지식 그래프` visible in the main change, then explain their value in plain language. Exclude implementation minutiae, not the identity of the feature.
- Present optionality as reassurance after introducing the new feature: `벡터 검색(의미 기반 검색) 기능 추가` followed by the user's choice to enable it. Do not give optional setup a separate headline that obscures the addition.
- Use specific scenarios to sell verified benefits. Avoid unsupported speed, completeness, reliability, or effortless-setup claims; keep material consent, storage, waiting-time, and compatibility limits clear.

## End-user relevance

- Include a change only when it changes the installed product, installation or update experience, user workflow, safety protection, or user-facing understanding of a usable feature.
- Exclude GitHub Release language order or formatting, CI and test procedures, release verification records, repository plans, source documentation, internal catalog or projection work, and contributor workflows unless the change directly alters a subscriber action or outcome.
- In particular, do not report an English-first and Korean-second GitHub Release description format. It is a publication detail for developers and contributors, not a subscriber improvement.
- Test every candidate bullet with this question: “What can the subscriber now do, avoid, or understand differently?” Omit it when there is no direct answer.

## Output rules

- Use Korean for the title and bullets unless the requester explicitly selects another output language.
- Use natural, confident Korean with readable sentences; do not copy terse internal checklist wording. For an agent's separate review, use `리뷰` rather than presenting a `검토자` as the feature.
- Every bullet must state the subscriber benefit or changed capability; do not expose implementation, process, or publication details as an improvement.
- Do not place the banner image or a webhook URL in the subscriber summary file.
- Preserve security boundaries. Describe blocked secrets or credentials only as a safety outcome; never include a secret-shaped example or value.
- State uncertainty when release evidence is missing or contradictory instead of inferring a claim.

## Scope boundary

This is an explicit source-project-only Skill. Keep it under `.agents/skills/update-summary/`. Do not add it to `harness/`, a product catalog, a release bundle, or a consumer project projection.
