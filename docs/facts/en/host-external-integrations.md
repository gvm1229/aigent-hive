---
schema_version: 1
pair_id: host-external-integrations
topic_slug: host-external-integrations
language: en
counterpart: ../ko/host-external-integrations.md
title: "Discord v0.9 and Notion v0.10 Host Integrations"
summary: "Discord v0.9 can safely select localized outbound usage-alert fields and preview the real alert format; Notion connection and host OAuth are deferred to the first v0.10 test release."
tags: [discord, integration, notion]
aliases: ["Host integration priority"]
sources:
  - "repo:crates/hive-cli/src/discord.rs#sha256:8084b804ff091920b2ed588c04d0fce46e617196f78a93ec2bdb01e358a0489c"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:6c5524ce66035bf0f7cb7fd3a5d59780c1b17f3e33cbf09126534774228d077a"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:3e05d7beb0322270572036bf73dac4854b5468d1092bb2d940baad4492ca0e55"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:9ad86748e6144d65c143194538ba95369258ea0b00336f1b81d347fe1ce87245"
  - "repo:docs/plans/active/discord-onboarding-v09.md#sha256:74902022f12fcd031e58603c5b1867268833e993dbf7624a5a3123a42b8c9d6f"
  - "repo:docs/plans/active/v0.10.0-notion-candidate.md#sha256:f863a6c59dde7c117e9b4b294cb0974e051ffca5970d830cfa75e50d9799dc4f"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:9ffa22cf14504ba7385135c1f62fdcb19bede32a0925ed72eb23fa8b96359eb5"
  - "repo:schemas/user-setup.schema.json#sha256:e83e5f318a5b6ffcc08cfe0898a2b6138512c6bfb0eea99c6070b134f3712f47"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:31f5c7616a14d63a68aee677a5b242ff5c5054e8"
status: active
---

# Discord v0.9 and Notion v0.10 Host Integrations

Discord outbound notifications support a bounded, ordered field list in the selected interface
language. The test message uses the real alert renderer, fields, and order; only its localized
first line says that the user may request a format change. A real halt reports safe project,
run-title/checklist progress, host, and remaining usage without a raw prompt, session ID,
absolute path, or credential. Global setup records only the webhook environment-variable name
and resumable non-secret answers. Notion remains internal to v0.10; v0.9 excludes its OAuth
tokens, webhook URLs, raw prompts, and absolute paths.
