---
schema_version: 1
pair_id: version-policy
topic_slug: version-policy
language: ko
counterpart: ../en/version-policy.md
title: "Version 정책"
summary: "Compatible feature의 minor, fix의 patch, exact user target 기반 major."
tags: [release, semver, version]
aliases: ["Version lifecycle"]
sources:
  - "repo:docs/decisions/ADR-0006-version-lifecycle.md#sha256:0cb27b7ca375a4e3646e0bb2a498052a9e266f72effde3b7659ed2e83f5f0298"
links: [release-verification, test-distribution]
reviewed_revision: "git:e072135e0148176a5a91159f60ad36ad82eabf73"
status: active
---

# Version 정책

Backward-compatible feature: exact next minor. Compatible fix: exact next patch.
Major: 사용자의 exact version과 별도 confirmation 전 inference·준비 금지.
