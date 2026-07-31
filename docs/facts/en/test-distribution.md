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
  - "repo:Cargo.toml#sha256:5083784d829c1e5ee6e642b54a3e616e78327dc1b8deb139bc00f8d14374b830"
  - "repo:scripts/package-npm.mjs#sha256:22c8a4e6b71764d2c3987a3525736d7406ce2a0d6da75ed96da420996a4d2e2c"
links: [global-onboarding, version-policy]
reviewed_revision: "git:b74afdae66f2704c6b24e42d47332ed931e2fecd"
status: active
---

# 0.8.0 Test Distribution

The product candidate is `0.8.0`, dated `2026-07-31`; npm transport starts at
`0.8.0-test.1` under `test`. No GitHub Release, stable npm `0.8.0`, or `latest`
move occurs. Artifacts originate from exact protected `develop`; after npm and
direct-installer qualification, the same commit reaches `main` by pull request.
Acceptance requires separate product/package inputs, exact manifests, matching
platform dependencies, and passing packaging/workflow tests. Origin: repeatable
installation tests requested before the maintainer approves a public release.
