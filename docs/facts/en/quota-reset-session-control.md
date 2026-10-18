---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: en
counterpart: ../ko/quota-reset-session-control.md
title: "Session-Bound Reset-Only Control"
summary: "One-shot stop, direct resume, Windows evidence and local monitoring."
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:fb83162490270dc48650dd60c0a93c321c8811915d863fd102e478f9d22967f4"
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:39d24941f041d49e4aaee3c9130d0e430a7a14a5328557701e223dc93299aa65"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:7c60015123b18b54f4b2a1984839d5a3e5fc96b4935b8ded0ba7a6367d41279d"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:3a1337362f351951f06db51fc89b8fa06eea5b458534a06a2dc45463c7766139"
  - "repo:docs/guides/installed-usage-guard.md#sha256:369a6681fcc7415c711ab7d5b9d1dae879b9d6fc188261af10a4ffcb5327a6f2"
  - "repo:docs/research/public-test-0.13.0.md#sha256:c831b239c282a19ee42a0cbf9779484f044bd973425d61f319e9c9947de7a2f9"
  - "repo:docs/research/release-0.13.0.md#sha256:fbdd1a5c8e306a8af4e54cc12306d9da35a41ba71ea7493adbc4de424be8f1a2"
  - "repo:docs/research/session-stop-0.12.1.md#sha256:50b5ba7baa1b9c2ed7bf96669115860655bb2fe0b7b5d7df7ac5fa628c650437"
  - "repo:docs/research/stable-release-0.13.0.md#sha256:75d8a1e6acf84fcb526d57d93166edfcfb52964591cf207fa733fe6ff3fa1b4b"
  - "repo:docs/research/usage-stop-notice-0.12.1.md#sha256:58148052d858dec443c5d8231c0ad91c7748dfd8e54e2a5b5939e6fc5790cffd"
  - "repo:tests/results/session-stop-one-shot-code-0.12.1.json#sha256:f97ceca1df94fff473e56240f8db49dce4ffe644b455154439008e67a3c21b56"
  - "repo:tests/results/session-stop-one-shot-installation-0.12.1.json#sha256:bbc233760b14fc02ebf35467618df2e59c26bab4e549806634a3c96d058d2371"
  - "repo:tests/results/session-stop-one-shot-qualification-0.12.1.json#sha256:000da296ad912dc0d30d30baed0f79f8a88193d6b09606bd75eff29e1dbe4406"
  - "repo:tests/results/session-stop-partial-windows-0.12.1.json#sha256:1f1d27dae9ef95f345c39fa018dce9f01b752cd765d24de3609b1ac7280a4071"
links: [automatic-dispatch-guard, installed-usage-guard, usage-stop-notice]
reviewed_revision: "git:38f6f050c1a8ba8261ef2045d59b5e4789c54ba9"
status: active
---

# Session-Bound Reset-Only Control

The opt-in 0.13.0 monitor stops local Windows tasks once on an observed reset or a 60s valid-reading gap. Poll starts are 15s apart, with 5s timeouts and 2s retries. No resume lock or acknowledgement exists; direct resume and new input remain possible. History prevents replay after restart; preflight deduplicates monitor-handled resets. Threshold and zero checks remain. The qualified 0.12.1-dev prototype proves four real native stops in 283–306ms with the app open, 4/4 reads, one-shot regressions and local always-on sensing. The same control/notice code and public test.1 pass three-OS package checks. Other OS, remote/cloud interruption and provider-refill/server-billing cutoff remain unproven.
