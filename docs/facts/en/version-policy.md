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
  - "repo:docs/decisions/ADR-0006-version-lifecycle.md#sha256:b0f4b815c2842d969297db6783ecbc12330d624f4efb853c2da8cf662dc501f7"
links: [release-verification, test-distribution]
reviewed_revision: "git:9170c884c9c96d99abcea1f5ab96a4a3a62541be"
status: active
---

# Version Policy

A backward-compatible feature requires the exact next minor version and a compatible
fix the exact next patch. Hive never infers or prepares a major target without the
user's exact version and separate confirmation.
