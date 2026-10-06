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
  - "repo:.agents/skills/update-summary/SKILL.md#sha256:73341fa979bd74aedda660ba44f68dba7a1137702e4bec0922bfa3ebd17a9348"
  - "repo:docs/archive/plans/foundations/source-update-summary-skill.md#sha256:4c2eb48e174ddacef78f3b1d576db2f703f4807632feac925458128da4dd9039"
  - "repo:docs/releases/0.10.0.subscriber.ko.md#sha256:ce658d7a5addabc93d69c99d3bea80fd0137c61d3141c9880c05fa1e50d4e426"
  - "repo:scripts/register-stable-summary-approval.py#sha256:8cd05c881ecadb7324bb144b0ff20e9c1a3629e6386bcce4d31a99d86c8e6c10"
links: [public-skill-identity, source-development, v0-9-full-release]
reviewed_revision: "git:2d59ff6bd00e1f7c3f44d0744700a736bfdb7a3d"
status: active
---

# Source-Project Update Summary Skill

Nonshipping `update-summary` compares requested releases for readers unfamiliar with Hive. Describe verified complete benefits, examples, choices, costs, limits and core technical names; keep failed checks internal. New copy uses ordered new-feature/fix/improvement sections with `---` between populated sections; omit empty categories and assign each change once. Use approved 0.10.0 as a wording reference, not a layout. Count the whole message toward 2,000 characters; shorten only above that limit. Drafts are unissued; wording approval is not release authority. Register approved digests with existing gh, without manual setup or sending. Compare file/sidecar/external digest without refreshing; changed copy needs new approval, retries reuse it.
