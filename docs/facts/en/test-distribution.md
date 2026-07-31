---
schema_version: 1
pair_id: test-distribution
topic_slug: test-distribution
language: en
counterpart: ../ko/test-distribution.md
title: "npm 0.8.0 Distribution"
summary: "Exact npm 0.8.0 is the latest install channel while GitHub Release and Git release tags remain absent."
tags: [distribution, release, test]
aliases: ["0.8.0 release scope"]
sources:
  - "repo:Cargo.toml#sha256:5083784d829c1e5ee6e642b54a3e616e78327dc1b8deb139bc00f8d14374b830"
  - "repo:scripts/package-npm.mjs#sha256:7d286e69158752940c877ce7b8604ee336b7beb7ede0632871d9bae2e9546710"
links: [global-onboarding, version-policy, windows-powershell-module-isolation]
reviewed_revision: "git:cdde668bed5f3b35e08a35f64e7e25594ce9c3a2"
status: active
---

# npm 0.8.0 Distribution

Exact npm `0.8.0` is published under `latest`; `npm install -g aigent-hive` and
exact `@0.8.0` resolve the same package family. The immutable
`0.8.0-test.1|test` distribution remains as prior validation history. No GitHub
Release or Git release tag is created. Artifacts originate from the exact
protected `develop` candidate, and successful npm qualification precedes the
same commit's pull request to `main`. Acceptance requires exact product/package
version equality, matching platform dependencies, provenance, byte identity,
and packaging/workflow tests. Origin: the maintainer requested the untagged
`0.8.0` npm release when npm required a valid `latest` channel.
