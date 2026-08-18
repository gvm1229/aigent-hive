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
  - "repo:docs/decisions/ADR-0006-version-lifecycle.md#sha256:b0f4b815c2842d969297db6783ecbc12330d624f4efb853c2da8cf662dc501f7"
links: [release-verification, test-distribution]
reviewed_revision: "git:9170c884c9c96d99abcea1f5ab96a4a3a62541be"
status: active
---

# Version 정책

Backward-compatible feature: exact next minor. Compatible fix: exact next patch.
Major: 사용자의 exact version과 별도 confirmation 전 inference·준비 금지.
