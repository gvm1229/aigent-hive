---
schema_version: 1
pair_id: host-external-integrations
topic_slug: host-external-integrations
language: en
counterpart: ../ko/host-external-integrations.md
title: "Discord v0.9 and Notion v0.10 Host Integrations"
summary: "Discord v0.9 previews and delivers the same localized, sectioned Markdown usage alert; Notion connection and host OAuth are deferred to the first v0.10 test release."
tags: [discord, integration, notion]
aliases: ["Host integration priority"]
sources:
  - "repo:crates/hive-cli/src/discord.rs#sha256:8e46be8e49884c9fbfacee0b17c2588bd637ff08118e4d98465dbc7b45ccba77"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:3f107ad6b4ac75f191f2bc6933a60d14e1e194b2ed5f12376e433a8f11761b0c"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:cb42f6c3bd643bc236f3af89f4388ffdbc08db66af88123a38267b904d7b9d01"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:9ad86748e6144d65c143194538ba95369258ea0b00336f1b81d347fe1ce87245"
  - "repo:docs/plans/active/discord-onboarding-v09.md#sha256:91a27ed57ddd259ac0a3270ee9242243f0a567bdae3fc756b90f76303c01c037"
  - "repo:docs/plans/active/v0.10.0-notion-candidate.md#sha256:f863a6c59dde7c117e9b4b294cb0974e051ffca5970d830cfa75e50d9799dc4f"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:4f3676378fafac75f9c6376210c760a2e0200e843ead0825d1b34d7446864e34"
  - "repo:schemas/user-setup.schema.json#sha256:e83e5f318a5b6ffcc08cfe0898a2b6138512c6bfb0eea99c6070b134f3712f47"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:4c74e7b82263f85bee21a2272dc865eeb60eaa04"
status: active
---

# Discord v0.9 and Notion v0.10 Host Integrations

Discord outbound notifications support a bounded, ordered field list in the selected language.
Test and real alerts share a renderer, with a test notice before the real content. The real
content uses blank-line-separated Markdown sections with emoji and bold titles:
usage, task details, and a direct instruction for asking Hive to continue the task. It contains no
underlined text. A real halt reports safe project, run-title/checklist progress, host, and
remaining usage without a raw prompt, session ID, absolute path, or credential. Global setup
records only the webhook environment-variable name and resumable non-secret answers. Notion
remains internal to v0.10; v0.9 excludes its OAuth tokens, webhook URLs, raw prompts, and
absolute paths.
