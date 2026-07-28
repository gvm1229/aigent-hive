---
schema_version: 1
pair_id: boundaries
topic_slug: boundaries
language: en
counterpart: ../ko/boundaries.md
title: "Ownership and Orchestration Boundaries"
summary: "Separation of source, release, consumer ownership, and replaceable orchestration dependencies."
tags: [boundaries, orchestration, ownership]
aliases: ["Hive ownership boundaries"]
sources:
  - "repo:AGENTS.md#sha256:8293c7e01a78bbf6106fc6ee9cca9748171ba2361c5003883ad11faa4a81b396"
  - "repo:docs/decisions/ADR-0001-source-release-installed-boundary.md#sha256:51850d51887f4d2cd4759e562aedee458398463e2b219cb94ca7b4540ad5bab7"
  - "repo:docs/decisions/ADR-0011-source-wiki-independence.md#sha256:15dbcb1c9e294078dc641d0c51c3655bd047cdf1c57629cb4158e7d047097f1b"
links: [knowledge, plugin-lifecycle, source-overview]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# Ownership and Orchestration Boundaries

Hive keeps three artifact classes separate: this source workspace, immutable release bundles, and
installed consumer harnesses. Source directives never ship to consumers. Installed user state,
runtime data, and project knowledge never flow back into source. Hive mutates only declared owned
paths or exact owned marker blocks and preserves user-authored and third-party bytes.

## Why the OMX Wiki Skill is excluded

- Current role: OMX on Codex and OMC on Claude remain active source-development orchestration aids.
- Exclusion: the fixed `omx_wiki/` storage path, `omx wiki` command surface, and
  `.omx-config.json` lifecycle would couple durable Hive-owned knowledge to replaceable tooling.
- Ownership: canonical source knowledge remains under Hive's provider-neutral `llm-wiki/`
  contract, with SQLite only as a derived local index.
- Retirement condition: replacing OMX/OMC must be a tooling change only, with zero source-knowledge
  migration.

This exclusion is about lifecycle ownership, not the quality or usefulness of OMX Wiki. OMX/OMC
may assist execution while Hive's durable data, paths, schemas, and Skill identities remain
independent of their namespaces.
