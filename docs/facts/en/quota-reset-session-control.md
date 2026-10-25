---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: en
counterpart: ../ko/quota-reset-session-control.md
title: "Session-Bound Reset-Only Control"
summary: "0.13.1 reset comparison excludes expired observations and prior-process baselines; floor and zero-quota protection remain."
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:9b858bef8831bed659b0f977ecd9ba84111800176d9dfc048b1d46c4c6ab8115"
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:ad6bc77e2e15a0cb969c7066929d8840d90ad6fbce309f526f43100ba31a52c2"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:6effcddaa5061365989f4ce449664b30f59bebc1eaf15f6fdf08f2d9e741a102"
  - "repo:crates/hive-core/src/usage_guard.rs#sha256:e59ef8b5bfe8b2024bb68a6940e23bf9ea45a4af611d7f5d2078f11c1cdfe228"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:212411004c3dcfe328442a665fa411d7ab2a942baa0b654aacc38b585c54d55a"
  - "repo:docs/guides/installed-usage-guard.md#sha256:0ead68f33595fcbd18d4d9c9ec38926645ca3cf22ffd895a02628c5e3c97a384"
  - "repo:docs/research/public-test-0.13.0.md#sha256:c831b239c282a19ee42a0cbf9779484f044bd973425d61f319e9c9947de7a2f9"
  - "repo:docs/research/release-0.13.0.md#sha256:fbdd1a5c8e306a8af4e54cc12306d9da35a41ba71ea7493adbc4de424be8f1a2"
  - "repo:docs/research/session-stop-0.12.1.md#sha256:50b5ba7baa1b9c2ed7bf96669115860655bb2fe0b7b5d7df7ac5fa628c650437"
  - "repo:docs/research/stable-release-0.13.0.md#sha256:75d8a1e6acf84fcb526d57d93166edfcfb52964591cf207fa733fe6ff3fa1b4b"
  - "repo:docs/research/usage-preflight-baseline-0.13.1.md#sha256:28e926a63e9f99f22b8c1331956d0bdb12959af37d29c867cb7234e1ed534761"
  - "repo:docs/research/usage-stop-notice-0.12.1.md#sha256:58148052d858dec443c5d8231c0ad91c7748dfd8e54e2a5b5939e6fc5790cffd"
  - "repo:tests/results/session-stop-one-shot-code-0.12.1.json#sha256:f97ceca1df94fff473e56240f8db49dce4ffe644b455154439008e67a3c21b56"
  - "repo:tests/results/session-stop-one-shot-installation-0.12.1.json#sha256:bbc233760b14fc02ebf35467618df2e59c26bab4e549806634a3c96d058d2371"
  - "repo:tests/results/session-stop-one-shot-qualification-0.12.1.json#sha256:000da296ad912dc0d30d30baed0f79f8a88193d6b09606bd75eff29e1dbe4406"
  - "repo:tests/results/session-stop-partial-windows-0.12.1.json#sha256:1f1d27dae9ef95f345c39fa018dce9f01b752cd765d24de3609b1ac7280a4071"
  - "repo:tests/results/usage-preflight-baseline-0.13.1.json#sha256:863442407f17d14fe0f89fd5c296ca8b7214361b3916b10820320df12b164c7e"
links: [automatic-dispatch-guard, installed-usage-guard, usage-stop-notice]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Session-Bound Reset-Only Control

The opt-in 0.13.0 monitor stops local Windows tasks once on an observed reset or a 60s valid-reading gap. Poll starts are 15s apart, with 5s timeouts and 2s retries. No resume lock or acknowledgement exists; direct resume and new input remain possible. History prevents replay after restart; preflight deduplicates monitor-handled resets. Threshold and zero checks remain. The qualified 0.12.1-dev prototype proves four real native stops in 283–306ms with the app open, 4/4 reads, one-shot regressions and local always-on sensing. The same control/notice code and public test.1 pass three-OS package checks. Other OS, remote/cloud interruption and provider-refill/server-billing cutoff remain unproven.
