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
  - "repo:docs/research/codex-resume-lock-reference.md#sha256:33c7e3ebaa5b92c006a52e7ea57f91aea3ac948b0af221aaa0e412d63ef6d0c9"
  - "repo:docs/research/stable-release-0.13.0.md#sha256:75d8a1e6acf84fcb526d57d93166edfcfb52964591cf207fa733fe6ff3fa1b4b"
  - "repo:tests/results/session-adapter-latch-windows-0.12.1.json#sha256:f22b0a6a65fe3d2197972ef342a59fc67081c9dfb7c0dcd649c61db1672b0259"
links: [quota-reset-session-control, usage-stop-notice]
reviewed_revision: "git:38f6f050c1a8ba8261ef2045d59b5e4789c54ba9"
status: active
---

# Historical Codex Resume Lock

The historical lock combined a durable incident, Hive dispatch refusal and one-second native user-stop requests. Resume started another turn that the monitor interrupted again. The source-only reference pins the old commit, a bounded single-thread reproduction, exact incident acknowledgement with fresh sensing, and its limits. A scoped new-turn confirmation took 136ms with the app preserved. The 0.12.1-dev prototype and 0.13.0 release use one-shot interruption and a close-only notice; it has no resume lock. The request preserves the unwanted mechanism for future reference, not activation.
