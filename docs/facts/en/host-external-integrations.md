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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:b2d3c7a9a42ce53e2ab8806401efb6e7076d7550843f0a09dd7158de56eee08f"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:e1e23470bdd37528700da00641cedef9e510c525863c991c9d189ab761e9bde1"
  - "repo:docs/archive/plans/foundations/v0.10.0-notion-candidate.md#sha256:f863a6c59dde7c117e9b4b294cb0974e051ffca5970d830cfa75e50d9799dc4f"
  - "repo:docs/archive/plans/releases/0.9.0/discord-onboarding-v09.md#sha256:91a27ed57ddd259ac0a3270ee9242243f0a567bdae3fc756b90f76303c01c037"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:dff20a5844cfaa8a4958ea1755392f0c598c77fc6679e72e275de7249deb3c87"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
  - "repo:schemas/user-setup.schema.json#sha256:9c4d51829f1c6bc8553ed566a9663907dd5746d08b94a3a76c7744e21b556548"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:72bd93df896956e10c30013db3bc9232b7a4a3ca"
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
