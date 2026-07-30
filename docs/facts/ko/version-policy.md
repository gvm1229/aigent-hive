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
  - "repo:docs/decisions/ADR-0006-version-lifecycle.md#sha256:b314d07c19558eb0de0b629250ea19c4ede782f4afc66d92078e9660f75eb26e"
links: [release-verification, test-distribution]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Version 정책

Backward-compatible feature: exact next minor. Compatible fix: exact next patch.
Major: 사용자의 exact version과 별도 confirmation 전 inference·준비 금지.
