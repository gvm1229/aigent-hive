---
schema_version: 1
pair_id: codex-resume-lock-reference
topic_slug: codex-resume-lock-reference
language: en
counterpart: ../ko/codex-resume-lock-reference.md
title: "Historical Codex Resume Lock"
summary: "Repeat interruption reference, excluded from the current release."
tags: [codex, interruption, reference]
aliases: []
sources:
  - "repo:docs/research/codex-resume-lock-reference.md#sha256:1770e8379d94ab3ceae4c08c85e403ede1dba93c521e2ca7c88d3b96017bf3e9"
  - "repo:tests/results/session-adapter-latch-windows-0.12.1.json#sha256:f22b0a6a65fe3d2197972ef342a59fc67081c9dfb7c0dcd649c61db1672b0259"
links: [quota-reset-session-control, usage-stop-notice]
reviewed_revision: "git:6cee0fc8998d4b1b25eaed092f1a252b7144b210"
status: active
---

# Historical Codex Resume Lock

The historical lock combined a durable incident, Hive dispatch refusal and one-second native user-stop requests. Resume started another turn that the monitor interrupted again. The source-only reference pins the old commit, a bounded single-thread reproduction, exact incident acknowledgement with fresh sensing, and its limits. A scoped new-turn confirmation took 136ms with the app preserved. The 0.12.1-dev prototype and 0.13.0 target use one-shot interruption and a close-only notice; it has no resume lock. The request preserves the unwanted mechanism for future reference, not activation.
