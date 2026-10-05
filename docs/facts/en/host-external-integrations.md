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
  - "repo:crates/hive-cli/src/discord.rs#sha256:8e46be8e49884c9fbfacee0b17c2588bd637ff08118e4d98465dbc7b45ccba77"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:b96d727dc4effe4c4ed77141927a7a67b4a5be047683544c4992b1e274b9ef55"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ffaa44da03bc2179a4b9d7e743fb42d556506338072fe7de2d73dc3100409971"
  - "repo:docs/archive/plans/foundations/v0.10.0-notion-candidate.md#sha256:f863a6c59dde7c117e9b4b294cb0974e051ffca5970d830cfa75e50d9799dc4f"
  - "repo:docs/archive/plans/releases/0.9.0/discord-onboarding-v09.md#sha256:91a27ed57ddd259ac0a3270ee9242243f0a567bdae3fc756b90f76303c01c037"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:160bc8bc434f1547e1fb3dad23902b740304323c800b92ece62ccda10a61114e"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
  - "repo:schemas/user-setup.schema.json#sha256:9c4d51829f1c6bc8553ed566a9663907dd5746d08b94a3a76c7744e21b556548"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:e22ff9036b04535dc9a9cc542f08b20143cfe40e"
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
