---
schema_version: 1
pair_id: plugin-lifecycle
topic_slug: plugin-lifecycle
language: en
counterpart: ../ko/plugin-lifecycle.md
title: "User Plugin Lifecycle"
summary: "Native plugin installation, discovery, guidance append, and host-specific ownership for three supported hosts."
tags: [hosts, installation, plugins]
aliases: ["user plugin lifecycle"]
sources:
  - "repo:crates/hive-cli/src/user_install.rs#sha256:ea61dbde5664499d96bb895b391c445d822dd7373f8e9c6daa1ee372efa3e90d"
  - "repo:docs/research/user-plugin-host-surfaces.md#sha256:d5fa0cac4d0aebe9ae08c966d16dc8428c9b1dae65a816a2a9500617ffe3e2f6"
  - "repo:harness/plugins/aigent-hive/plugin.json#sha256:2eeb1a2cb0d4f2c616443e1b5844b1e10551457f78f1cb96ff76afb223495e86"
links: [boundaries, skill-routing, upgrade]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# User Plugin Lifecycle

Hive uses each host's native plugin package and discovery surface. Codex installs from a local
marketplace, Claude Code uses its user-scoped marketplace contract, and Antigravity validates and
installs a root `plugin.json` package through `agy`.

User-level guidance is appended through one exact `AIGENT-HIVE:USER` marker block. Codex targets
the active global AGENTS file, Claude targets `~/.claude/CLAUDE.md`, and Antigravity targets
`~/.gemini/GEMINI.md`. Existing OMX, OMC, and foreign bytes remain outside Hive ownership.

Install, update, validate, and recover operate through a pinned user-root capability with bounded
host commands and authenticated inventories. Codex and Antigravity have local qualification
evidence at the recorded versions. Claude's contract is documented, while live CLI qualification
remains an external protected-environment gap.
