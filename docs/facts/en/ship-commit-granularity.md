---
schema_version: 1
pair_id: ship-commit-granularity
topic_slug: ship-commit-granularity
language: en
counterpart: ../ko/ship-commit-granularity.md
title: "Ship Commit Granularity"
summary: "The product ship Skill treats all-changes requests as scope, not as permission for an aggregate commit, and commits each independently reviewable and revertible concern separately."
tags: [commit, git, skill, workflow]
aliases: ["Atomic ship commits"]
sources:
  - "repo:docs/skills.md#sha256:b1d168024659e23bc1fee30c46e2b628e607522b9b0da2f59229a277eff2a702"
links: [public-skill-identity, source-development]
reviewed_revision: "git:23dafb9d646ea893ce06f6ec2cc9ea22b7eed673"
status: active
---

# Ship Commit Granularity

The product `ship` Skill reads the repository Git rules, inspects the full worktree, and maps
each independently reviewable and revertible concern to its files or hunks, nearest verification,
and proposed commit before staging. A request covering all files or all changes authorizes the
complete concern set; it does not authorize a single aggregate commit.

The Skill stages and verifies one concern at a time, refreshes the concern map after every commit,
uses patch staging when one file contains multiple concerns, and leaves ambiguous ownership or
scope untouched. It preserves repository hook, history, and explicit-push boundaries.
