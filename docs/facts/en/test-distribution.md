---
schema_version: 1
pair_id: test-distribution
topic_slug: test-distribution
language: en
counterpart: ../ko/test-distribution.md
title: "0.8.0 Test Distribution"
summary: "Product candidate 0.8.0 uses npm package versions 0.8.0-test.N without a GitHub Release or npm latest."
tags: [distribution, release, test]
aliases: ["0.8.0 release scope"]
sources:
  - "repo:docs/decisions/ADR-0013-0.8-release-scope.md#sha256:a958146a8ea6d747fa485cf5ba0ec81f0471567723589fe82a6aa90815cece06"
  - "repo:scripts/package-npm.mjs#sha256:22c8a4e6b71764d2c3987a3525736d7406ce2a0d6da75ed96da420996a4d2e2c"
links: [global-onboarding, version-policy]
reviewed_revision: "git:3143c0e90b3c474c739651f7ddc2350bbf5e020a"
status: active
---

# 0.8.0 Test Distribution

The product candidate remains exact `0.8.0`, while npm transport versions use
`0.8.0-test.N`; the first candidate is `0.8.0-test.1`. The `test` tag makes it
installable as `aigent-hive@test` without consuming stable npm `0.8.0`, creating a
GitHub Release, or moving npm `latest`. Candidate artifacts must originate from the
exact protected `develop` commit. After successful npm and direct-installer
qualification, that same commit moves to `main` through a pull request. Acceptance:
separate product and package inputs, exact manifest metadata, matching platform
dependency versions, and passing packaging and release-workflow tests. Origin: the
maintainer requested repeatable pre-release installation tests before approving the
actual public release.
