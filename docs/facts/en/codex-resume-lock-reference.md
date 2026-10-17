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
  - "repo:docs/research/codex-resume-lock-reference.md#sha256:bb99293eb9bb3a588f91783c04eb2c8eb63479097802ac87a3d548fb9a79c87b"
  - "repo:tests/results/session-adapter-latch-windows-0.12.1.json#sha256:f22b0a6a65fe3d2197972ef342a59fc67081c9dfb7c0dcd649c61db1672b0259"
links: [quota-reset-session-control, usage-stop-notice]
reviewed_revision: "git:fd73d81bf5078662a431bcf1712b579a85c2b0cb"
status: active
---

# Historical Codex Resume Lock

The historical lock combined a durable incident, Hive dispatch refusal and one-second native user-stop requests. Resume started another turn that the monitor interrupted again. The source-only reference pins the old commit, a bounded single-thread reproduction, exact incident acknowledgement with fresh sensing, and its limits. A scoped new-turn confirmation took 136ms with the app preserved. Current 0.12.1 uses one-shot interruption and a close-only notice; it has no resume lock. The request preserves the unwanted mechanism for future reference, not activation.
