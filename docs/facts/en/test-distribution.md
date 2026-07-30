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
  - "repo:docs/decisions/ADR-0013-0.8-release-scope.md#sha256:2fb97b133d567155c0f333cbe7a401fc7473e849d88db2e2f9b897d7acecb39e"
links: [global-onboarding, version-policy]
reviewed_revision: "git:99f39edd08cc4b9d513f073d297bed05e2772c9d"
status: active
---

# 0.8.0 Test Distribution

Exact `0.8.0` is an install and update test distributed through npm exact version and
the `test` tag. It creates no GitHub Release, release tag, or npm `latest` movement.
The umbrella tarball carries rendered direct installers that verify the exact scoped
platform-package digest before installing its native binary. Candidate activation and
publication require an explicitly selected protected branch.
