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
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:6606c09b03b9a0b3896a8b9242a937aec0a25a644ffbf873a3117e6c47410ccf"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:9ad86748e6144d65c143194538ba95369258ea0b00336f1b81d347fe1ce87245"
  - "repo:docs/plans/active/discord-onboarding-v09.md#sha256:91a27ed57ddd259ac0a3270ee9242243f0a567bdae3fc756b90f76303c01c037"
  - "repo:docs/plans/active/v0.10.0-notion-candidate.md#sha256:f863a6c59dde7c117e9b4b294cb0974e051ffca5970d830cfa75e50d9799dc4f"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:90a8ecca713a1b1963b5f1863f76d32d5c5b9532ca72922c2705ee9b63520307"
  - "repo:schemas/user-setup.schema.json#sha256:57a426a58c822271f1c6297c2c607e532e83c5652ca92ef68bdbcd8b95d357fd"
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
