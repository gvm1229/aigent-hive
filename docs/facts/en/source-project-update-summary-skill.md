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
  - "repo:.agents/skills/update-summary/SKILL.md#sha256:37f8bc71e3ef0e02ccaa82fc2d430dfb61be148f26be1709ac439c0424b8daf8"
  - "repo:docs/archive/plans/foundations/source-update-summary-skill.md#sha256:4c2eb48e174ddacef78f3b1d576db2f703f4807632feac925458128da4dd9039"
  - "repo:docs/decisions/subscriber-approval-via-main-2026-11-12.md#sha256:132fde7fd96e010174e23d95f237053f5bfc425c10e7906875ac473c51a311d1"
  - "repo:docs/releases/0.10.0.subscriber.ko.md#sha256:ce658d7a5addabc93d69c99d3bea80fd0137c61d3141c9880c05fa1e50d4e426"
  - "repo:tests/results/discord-sectioned-notification-0.12.0.md#sha256:40aaa2a04ecd3d673e28b09be83c2fb4a81f82f0297d931cd9eea407131077d1"
links: [public-skill-identity, source-development, v0-9-full-release]
reviewed_revision: "git:0bcae67aa54b63c6d52b58ee959bdb4425fcbe82"
status: active
---

# Source-Project Update Summary Skill

Nonshipping update-summary compares requested releases for newcomers: verified benefits, technical names, examples, choices, costs and limits. Keep failed checks internal. Noun phrases; no polite/mechanical endings. Tone: 0.11.1; positioning: approved 0.10.0. Ordered new/fix/improvement sections, separators, no empty category or duplicate. Whole 2,000-character limit; shorten only over limit. Drafts unissued. Reviewed develop-to-main PR approves copy/SHA-256; one-time environment variable enables sends. Validate released bytes/filename. Changed copy needs new PR/release source. No per-release registration; PR review alone grants no publication/delivery. Details: release guide and 2026-11-12 decision.
