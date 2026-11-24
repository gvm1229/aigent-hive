---
schema_version: 1
pair_id: host-external-integrations
topic_slug: host-external-integrations
language: en
counterpart: ../ko/host-external-integrations.md
title: "Discord Integration and Cancelled Notion Proposal"
summary: "Discord notifications remain supported; the Notion canonical backend proposal is cancelled."
tags: [discord, integration, notion]
aliases: ["Host integration priority"]
sources:
  - "repo:crates/hive-cli/src/discord.rs#sha256:069e81e74cb1ff8c3437cb4ffa1c008e8b791d12a7b8160af820ab34d0737d0d"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:50bbe234dff657b08dca42bc8cb17330a75d6aee436b5debd318fb039fd787c4"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ecb5eb8be00aa5902b1cf2d11d9bcdb51e76ceab68fce047c5463a9fc5920762"
  - "repo:docs/archive/plans/foundations/v0.10.0-notion-candidate.md#sha256:f863a6c59dde7c117e9b4b294cb0974e051ffca5970d830cfa75e50d9799dc4f"
  - "repo:docs/archive/plans/releases/0.9.0/discord-onboarding-v09.md#sha256:91a27ed57ddd259ac0a3270ee9242243f0a567bdae3fc756b90f76303c01c037"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:160bc8bc434f1547e1fb3dad23902b740304323c800b92ece62ccda10a61114e"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:005f05fa4631834f57fd4e8524e51a839d030e88ad1cf193c831a176fce03848"
  - "repo:schemas/user-setup.schema.json#sha256:f47704e89efc256c1ee46052bb594948d99cb5fcb19eb49628105f6c074ce71a"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Discord Integration and Cancelled Notion Proposal

Discord test and real usage alerts share a localized Markdown renderer. A test notice precedes
the real content. Sections cover usage, safe task details, and a resume instruction, with blank
lines, emoji, and bold titles. Real alerts report safe project and progress information without
raw prompts, session identifiers, private paths, or credentials. Setup stores the webhook
environment-variable name and resumable non-secret answers.

The maintainer cancelled the Notion canonical backend proposal on 2026-10-05. Its old design
remains historical reference, not an upcoming v0.10 feature. This cancellation does not remove
the completed Discord integration or authorize deletion of existing product data.
