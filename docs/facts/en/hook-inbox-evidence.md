---
schema_version: 1
pair_id: hook-inbox-evidence
topic_slug: hook-inbox-evidence
language: en
counterpart: ../ko/hook-inbox-evidence.md
title: "Hook inbox evidence boundary"
summary: "Worker execution and interactive Claude feedback need separate evidence."
tags: [evidence, hooks, orchestration]
aliases: [hook-inbox-evidence]
sources:
  - "repo:docs/research/overhaul/OV150-001.md#sha256:3d4b1d642c7c3dfe3be45e8f4df4191994bdbe8832a586b729370eaa108192d9"
links: [orchestration-ownership]
reviewed_revision: "git:0c4389da02bdf13f92b43878ab9eacd38c45396c"
status: active
---

# Hook inbox evidence boundary

Windows 11 with interactive Claude Code 2.1.322 and subscription-authenticated Codex completed OV150-001: Stop, one worker call, result and receipt, then matching Claude feedback. Replay caused no extra call but returned no receipt; OV160-002/003 own that gap. Gemini, other OSes and real-host cancellation/reclaim remain unverified.

Synthetic hook inputs alone prove only the handler leg. Queue files do not prove Stop or dispatch. Output-free Stop must keep pending work recoverable. The 0.15.0 product stays disabled with no activation path.
