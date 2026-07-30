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
  - "repo:docs/decisions/ADR-0006-version-lifecycle.md#sha256:b314d07c19558eb0de0b629250ea19c4ede782f4afc66d92078e9660f75eb26e"
links: [release-verification, test-distribution]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Version Policy

A backward-compatible feature requires the exact next minor version and a compatible
fix the exact next patch. Hive never infers or prepares a major target without the
user's exact version and separate confirmation.
