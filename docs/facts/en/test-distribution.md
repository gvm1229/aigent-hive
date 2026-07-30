---
schema_version: 1
pair_id: test-distribution
topic_slug: test-distribution
language: en
counterpart: ../ko/test-distribution.md
title: "0.8.0 Test Distribution"
summary: "Version 0.8.0 publishes only to npm test without a GitHub Release or npm latest."
tags: [distribution, release, test]
aliases: ["0.8.0 release scope"]
sources:
  - "repo:docs/decisions/ADR-0013-0.8-release-scope.md#sha256:7631f6a1b322510cf6b9b1d6e826681362cb494d250cf7137b4a29c446402b35"
links: [global-onboarding, version-policy]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# 0.8.0 Test Distribution

Exact `0.8.0` is an install and update test distributed through npm exact version and
the `test` tag. It creates no GitHub Release, release tag, or npm `latest` movement.
