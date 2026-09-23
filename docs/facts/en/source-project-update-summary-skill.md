---
schema_version: 1
pair_id: source-project-update-summary-skill
topic_slug: source-project-update-summary-skill
language: en
counterpart: ../ko/source-project-update-summary-skill.md
title: "Source Project-Only Update Summary Skill"
summary: "The source-only update-summary Skill promotes verified product changes, distinguishes new features from improvements, and keeps core technical names and practical user benefits visible."
tags: [development, release-notes, skill]
aliases: ["update-summary"]
sources:
  - "repo:.agents/skills/update-summary/SKILL.md#sha256:ca4bbfd9f46d7e92b9bdd3bc388935236c69e07730689ee8735ad068b0c56f0e"
  - "repo:docs/archive/plans/foundations/source-update-summary-skill.md#sha256:4c2eb48e174ddacef78f3b1d576db2f703f4807632feac925458128da4dd9039"
  - "repo:docs/releases/0.10.0.subscriber.ko.md#sha256:ce658d7a5addabc93d69c99d3bea80fd0137c61d3141c9880c05fa1e50d4e426"
  - "repo:scripts/register-stable-summary-approval.py#sha256:8cd05c881ecadb7324bb144b0ff20e9c1a3629e6386bcce4d31a99d86c8e6c10"
links: [public-skill-identity, source-development, v0-9-full-release]
reviewed_revision: "git:3a0d9e2e61d1867e0f38d8855ae8b064fa449f09"
status: active
---

# Source Project-Only Update Summary Skill

`update-summary` is source-only, excluded from product bundles and projections. It promotes
verified new, improved, fixed, and renamed features with core technical names, benefits, examples,
choices, and costs. Unreleased copy is a draft; wording approval is not release authority.
The approved 0.10.0 note (2026-09-01) is the required reference. After wording approval, the agent
runs `register-stable-summary-approval.py` with the approved digest and existing `gh` access.
No per-release manual GitHub setup. Changed wording needs new approval; retries reuse the same
approval. Delivery checks the file, sidecar, and external digest without refreshing any of them.

Assume no Hive internals knowledge. Lead with user benefits, not engineering.
