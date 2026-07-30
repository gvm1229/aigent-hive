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
  - "repo:docs/decisions/ADR-0013-0.8-release-scope.md#sha256:0a517849c6db000119e9c677b25304ce87a167b094473990dd9a1bb60ab609b6"
links: [global-onboarding, version-policy]
reviewed_revision: "git:cf992996d3076479bdfb433c4171eee046f571ae"
status: active
---

# 0.8.0 Test Distribution

Exact `0.8.0` is an install and update test distributed through npm exact version and
the `test` tag. It creates no GitHub Release, release tag, or npm `latest` movement.
The umbrella tarball carries rendered direct installers that verify the exact scoped
platform-package digest before installing its native binary.
