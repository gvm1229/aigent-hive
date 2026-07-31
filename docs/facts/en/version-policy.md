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
  - "repo:docs/decisions/ADR-0006-version-lifecycle.md#sha256:0cb27b7ca375a4e3646e0bb2a498052a9e266f72effde3b7659ed2e83f5f0298"
links: [release-verification, test-distribution]
reviewed_revision: "git:e072135e0148176a5a91159f60ad36ad82eabf73"
status: active
---

# Version Policy

A backward-compatible feature requires the exact next minor version and a compatible
fix the exact next patch. Hive never infers or prepares a major target without the
user's exact version and separate confirmation.
