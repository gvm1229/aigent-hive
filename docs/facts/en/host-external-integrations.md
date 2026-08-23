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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:6c5febe7ae1ac1a892f7ac412c40d1b8d9ae339fe73fa8153faf9bb22051e1c0"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:64e7ee1eb9aaafd399fe971ca35e5df6aee68285029a9b84fa6b928a3324ffdc"
  - "repo:docs/archive/plans/foundations/v0.10.0-notion-candidate.md#sha256:f863a6c59dde7c117e9b4b294cb0974e051ffca5970d830cfa75e50d9799dc4f"
  - "repo:docs/archive/plans/releases/0.9.0/discord-onboarding-v09.md#sha256:91a27ed57ddd259ac0a3270ee9242243f0a567bdae3fc756b90f76303c01c037"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:dff20a5844cfaa8a4958ea1755392f0c598c77fc6679e72e275de7249deb3c87"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:1fcbb2b9b2db6d57bd40682f80db2a0a916ebbffb3434431038b609b6b743c11"
  - "repo:schemas/user-setup.schema.json#sha256:06ed2a954a7c98817a1288a29f779c1db45cfafa2ea21d8227695a1d988b5fb6"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:47d4663f1e1f263276f9ce54b7c69a3ff95d2170"
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
