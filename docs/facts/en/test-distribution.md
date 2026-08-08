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
  - "repo:docs/releases/0.8.0.md#sha256:1d5100af5c1f8b2d9e19d2a730acdcff9d1fa276c0cfdb364ec9b33164b78205"
links: [global-onboarding, version-policy, windows-powershell-module-isolation]
reviewed_revision: "git:e37de7ff99fb235f673a4d3273deb54d6284999e"
status: active
---

# npm 0.8.0 Distribution

Candidate run `30657669889` qualified protected `develop` commit `420e244`.
Publication run `30658188721` then published all six npm packages as exact
`0.8.0|latest`, while preserving immutable `0.8.0-test.1|test`. The npm and
Windows direct-install binaries share SHA-256
`sha256:330f4e0c8da5b6347400b9b16a9f76b2fb4f94406a2eacfe8c641367ca344ef9`.
No GitHub Release or Git release tag was created. Origin: the maintainer
approved exact `0.8.0` publication because npm installation needed a valid
`latest` channel.
