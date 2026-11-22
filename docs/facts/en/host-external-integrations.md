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
  - "repo:crates/hive-cli/src/discord.rs#sha256:1176cda4fab177cc37d395c7b182c1b854a94f99b8141b07d09898a48199e4ae"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:412578c5555c660f88806d3902109d71c4ba90e986e4cf277ac91131088da0dc"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:33b0a59a13b659d8b98b583e682316b8df27964f7efc798af34967c8411cfe9e"
  - "repo:docs/archive/plans/foundations/v0.10.0-notion-candidate.md#sha256:f863a6c59dde7c117e9b4b294cb0974e051ffca5970d830cfa75e50d9799dc4f"
  - "repo:docs/archive/plans/releases/0.9.0/discord-onboarding-v09.md#sha256:91a27ed57ddd259ac0a3270ee9242243f0a567bdae3fc756b90f76303c01c037"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:160bc8bc434f1547e1fb3dad23902b740304323c800b92ece62ccda10a61114e"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:005f05fa4631834f57fd4e8524e51a839d030e88ad1cf193c831a176fce03848"
  - "repo:schemas/user-setup.schema.json#sha256:cefe939bedb7f6645fd1851e086def9677e4a2f3f7774258d54dd203cc8b10fc"
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
