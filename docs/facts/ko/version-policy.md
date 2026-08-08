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
  - "repo:docs/decisions/ADR-0006-version-lifecycle.md#sha256:7a26e1637241be7935c14d13778f838c9b4959191749899510705af85f1810a8"
links: [release-verification, test-distribution]
reviewed_revision: "git:d0747ee7e1851b9edfa2066214e948d75e895ebd"
status: active
---

# Version 정책

Backward-compatible feature: exact next minor. Compatible fix: exact next patch.
Major: 사용자의 exact version과 별도 confirmation 전 inference·준비 금지.
