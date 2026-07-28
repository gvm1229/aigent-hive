---
schema_version: 1
pair_id: index
topic_slug: index
language: en
counterpart: ../ko/index.md
title: "Aigent Hive Source Wiki"
summary: "Navigation for the provider-neutral bilingual knowledge of the Aigent Hive source workspace."
tags: [entrypoint, navigation, source-wiki]
aliases: ["Hive source knowledge"]
sources:
  - "repo:AGENTS.md#sha256:8293c7e01a78bbf6106fc6ee9cca9748171ba2361c5003883ad11faa4a81b396"
  - "repo:hive-source.json#sha256:528b3c6a8f8614a38065144f2de9f3cd527474d5e4ec3f720acd6a27e60f2019"
links: [boundaries, crate-architecture, knowledge, marketing-deck, plugin-lifecycle, security-release, skill-routing, source-overview, upgrade, usage-hosts, workflow]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# Aigent Hive Source Wiki

This Wiki records durable knowledge for developing Aigent Hive itself. English pages live under
`llm-wiki/en/`; each has an exact Korean counterpart under `llm-wiki/ko/`. The tracked Markdown is
canonical. The local SQLite index is disposable and rebuildable.

## Reading map

- Start with `source-overview` and `boundaries` for purpose, ownership, and provider neutrality.
- Use `crate-architecture`, `knowledge`, and `skill-routing` for implementation structure.
- Use `marketing-deck` to resume the Hive overview presentation in LumaDeck.
- Use `plugin-lifecycle`, `upgrade`, and `usage-hosts` for host-facing lifecycle behavior.
- Use `security-release` and `workflow` for trust boundaries and maintainer practice.

This repository is a Hive source workspace, not a consumer project. Source directives, runtime
scratch data, and this Wiki are never shipped as an installed project harness.
