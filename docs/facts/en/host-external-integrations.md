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
  - "repo:crates/hive-cli/src/discord.rs#sha256:524edb6f4e70a64cef99ffdef0e8347275701836c49e5b8b155edd37242fa6bd"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:ad1c22fbacbfab22bd4120a94bad5cb10ebc45936e39c5b0f586f5d9a2467a92"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:9d0a796027dde450cfec2162ac1073305068aa4ac0e6351303f4976c1ad87f38"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:9ad86748e6144d65c143194538ba95369258ea0b00336f1b81d347fe1ce87245"
  - "repo:docs/plans/active/discord-onboarding-v09.md#sha256:1ef1d3b61747f317ae1e7fced6e5dd60a6a1a09a6295fcb793d52634ba4098e9"
  - "repo:docs/plans/active/v0.10.0-notion-candidate.md#sha256:f863a6c59dde7c117e9b4b294cb0974e051ffca5970d830cfa75e50d9799dc4f"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
  - "repo:harness/skills/configure/SKILL.md#sha256:abeb032e21d2576366025465d54080966767fb7e17cca57848acf093eaa83eaf"
  - "repo:schemas/user-setup.schema.json#sha256:46b360a9f91e154d1440e2997b56a964edd122383ccfc9b105b4e2ae4f8939f9"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:31f5c7616a14d63a68aee677a5b242ff5c5054e8"
status: active
---

# Discord v0.9 and Notion v0.10 Host Integrations

Discord outbound notifications support a bounded, ordered field list in the selected interface
language. The test message uses the real alert renderer, fields, and order; only its localized
first line says that the user may request a format change. The default includes remaining usage
and a safe project identity. Request content stays local, and canonical progress is truthfully
unavailable until `DIS9-005–006` complete. Notion stays internal to v0.10; v0.9 excludes its
OAuth tokens, webhook URLs, raw prompts, and absolute paths.
