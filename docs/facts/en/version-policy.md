---
schema_version: 1
pair_id: version-policy
topic_slug: version-policy
language: en
counterpart: ../ko/version-policy.md
title: "Version Policy"
summary: "Compatible features increment minor, fixes increment patch, and major needs an exact user target."
tags: [release, semver, version]
aliases: ["Version lifecycle"]
sources:
  - "repo:docs/decisions/ADR-0006-version-lifecycle.md#sha256:7a26e1637241be7935c14d13778f838c9b4959191749899510705af85f1810a8"
links: [release-verification, test-distribution]
reviewed_revision: "git:d0747ee7e1851b9edfa2066214e948d75e895ebd"
status: active
---

# Version Policy

A backward-compatible feature requires the exact next minor version and a compatible
fix the exact next patch. Hive never infers or prepares a major target without the
user's exact version and separate confirmation.
