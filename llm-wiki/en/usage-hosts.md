---
schema_version: 1
pair_id: usage-hosts
topic_slug: usage-hosts
language: en
counterpart: ../ko/usage-hosts.md
title: "Usage Guard and Host Sensors"
summary: "Native-first quota sensing, CodexBar fallback boundaries, and source-session enforcement."
tags: [guard, hosts, usage]
aliases: ["usage guard hosts"]
sources:
  - "repo:.agents/skills/hive-usage-guard/scripts/guard.py#sha256:9be7431e5f63d3bfbdcab93b902cb736cd5e13b59622d0817e576f738b1e6df1"
  - "repo:crates/hive-cli/src/usage.rs#sha256:5bd67c08505d00136738ed34751412aa37d7242e43ecb0fbb1c22b5c2f4c0fed"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:141e8070b475ee2b0d81e93a69217093e07af9a9ca61c16dcbb31f111ea1d0f4"
links: [plugin-lifecycle, security-release, workflow]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# Usage Guard and Host Sensors

The usage guard checks the configured inclusive remaining-quota threshold before automatic work
boundaries. It records only sanitized sensor identity, window, timing, and decision data. Account
payloads, provider credentials, and raw quota responses are outside durable state.

Sensor order is native first, then CodexBar only as a fallback. Codex uses the local app-server
rate-limit method. Claude can use an explicitly configured, sanitized status-line capture.
Antigravity has no qualified official structured output yet, so its native sensor remains
unsupported. CodexBar is fallback-only for all three providers and requires explicit consent before
installation.

The source-development Python watcher and boundary gate are separate from the shipped one-shot
dispatch guard. A transient `unknown` waits three seconds and retries once; a repeated short glitch
remains visible but creates no new halt marker. Confirmed quota exhaustion and filesystem, session,
or sensor-integrity failures remain fail-closed.
