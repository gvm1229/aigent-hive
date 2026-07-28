---
schema_version: 1
pair_id: plugin-lifecycle
topic_slug: plugin-lifecycle
language: ko
counterpart: ../en/plugin-lifecycle.md
title: "User Plugin Lifecycle"
summary: "세 지원 host의 native plugin 설치, discovery, guidance append와 ownership."
tags: [hosts, installation, plugins]
aliases: ["사용자 플러그인 수명주기"]
sources:
  - "repo:crates/hive-cli/src/user_install.rs#sha256:ea61dbde5664499d96bb895b391c445d822dd7373f8e9c6daa1ee372efa3e90d"
  - "repo:docs/research/user-plugin-host-surfaces.md#sha256:d5fa0cac4d0aebe9ae08c966d16dc8428c9b1dae65a816a2a9500617ffe3e2f6"
  - "repo:harness/plugins/aigent-hive/plugin.json#sha256:2eeb1a2cb0d4f2c616443e1b5844b1e10551457f78f1cb96ff76afb223495e86"
links: [boundaries, skill-routing, upgrade]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# User Plugin Lifecycle

Host native plugin package와 discovery surface 사용.

- Codex: local marketplace를 통한 plugin 등록
- Claude Code: user-scoped marketplace contract
- Antigravity: root `plugin.json` package의 `agy` validate·install

User-level guidance append 단위: exact `AIGENT-HIVE:USER` marker block 1개. Codex target은
active global AGENTS file, Claude target은 `~/.claude/CLAUDE.md`, Antigravity target은
`~/.gemini/GEMINI.md`. Existing OMX, OMC와 foreign bytes는 Hive ownership 밖에 유지.

Install·update·validate·recover: pinned user-root capability, bounded host command와 authenticated
inventory 기반. Codex와 Antigravity: 기록된 version의 local qualification evidence 보유.
Claude: documented contract 보유, live CLI qualification은 external protected-environment gap.
