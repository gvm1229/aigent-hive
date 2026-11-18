---
schema_version: 1
pair_id: supported-hosts
topic_slug: supported-hosts
language: en
counterpart: ../ko/supported-hosts.md
title: "Supported Agent Hosts"
summary: "Hive has adapters for Codex, Claude Code, and Gemini Antigravity."
tags: [host, support]
aliases: ["Host support"]
sources:
  - "repo:docs/overview/product.md#sha256:d319ff6c624af8aa01d94e2daeeb8c7f4c3b9ccd1a16c3fd7d03514d9ff69a4a"
  - "repo:docs/research/overhaul/OV150-001.md#sha256:3d4b1d642c7c3dfe3be45e8f4df4191994bdbe8832a586b729370eaa108192d9"
links: [product-purpose, usage-sensor-policy]
reviewed_revision: "git:5ac2241398eece800912f20c25e84bc13332304a"
status: active
---

# Supported Agent Hosts

Hive implements adapters for Codex, Claude Code, and Gemini Antigravity. Codex and Antigravity have live-host evidence; Claude installation remains fixture-qualified.

Windows 11 with interactive Claude Code 2.1.322 and Codex completed a subscription-backed lab hook round trip. This does not qualify the full product host lifecycle, cancellation or reclaim. Gemini CLI and worker round trips on other OSes remain unverified. The 0.15.0 orchestration is disabled with no enablement path.
