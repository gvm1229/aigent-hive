---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: en
counterpart: ../ko/v0-9-full-release.md
title: "Aigent Hive 0.9.0 Test and Stable Release"
summary: "test.15 remains accepted; develop CI and five-target native runtime qualification passed. PR #19 merged to protected main, while the stable candidate, external TUF authorization, and publication remain pending."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:505cc48a16b2ccc7ca7fe39fdaf47d7b851a19810cb75c784fdfe5a6717c5823"
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:2691a98d452eac2b566e97dcd34982c7ef283bf14b01cd8b76508e1c82782403"
  - "repo:docs/guides/signed-update-and-release.md#sha256:aa570e405dc1e568a79fe6291e30807db9e96b7805e570aede152fed4120f5a5"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:1e0cc991893bc5e2d87a145e81217e29157be155ccfcfb8a4589d9d87779186c"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:1b7ea99554fcf2e475cc77dcb1a3452a7805315f"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Stable `v0.9.0` is absent; accepted prerelease `test.15` keeps `latest=0.8.0`. Windows preserving
reinstall passed. Develop CI run `31430181535` passed 19 jobs, and native runtime run `31428720884`
passed five targets. Stable source now binds explicit macOS ad-hoc and Windows unsigned
evidence to a deterministic TUF request, safe extraction, protected rollback floor, production verifier,
and exact target bytes. Paid platform certificates are not gates. PR #19 merged to protected `main`.
A stable candidate, external 2-of-3 authorization, and publication approval remain pending.
