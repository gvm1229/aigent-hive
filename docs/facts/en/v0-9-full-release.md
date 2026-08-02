---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: en
counterpart: ../ko/v0-9-full-release.md
title: "Aigent Hive 0.9.0 Test and Stable Release"
summary: "The 0.9.0 release uses an independently publishable bare test channel before a separately authorized stable publication."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:9172a8fa815052211dac6f561775f47852f4fe86bd629cb02004bbf5e0e30acb"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:df7dcaa87728498e03902dd8de2f1225ec6ef475f8894ba8b8d9f49f7540425c"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:bf7c1d3e36cd94e8ee5f2a68d9f8ca5c4c9f9c87"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Default test identity: package `0.9.0-test`, npm `test`, and a GitHub prerelease;
`0.9.0-test.N` only when needed. Candidate `30771098518` from `6761f0b` passed five
native targets and the npm umbrella. Public test publication is blocked on
default-branch workflow registration; its failed dispatch made no npm, tag, or GitHub
Release mutation. A registration-only `main` PR/review precedes protected publication.
Test publication never changes `latest` or triggers stable publication. Test and stable
artifacts have identical features, defaults, and diagnostics: report preview/export,
`markdown|notion`, and optional Discord guard notification. Stable follows test
acceptance through separate protected `main` publication.
