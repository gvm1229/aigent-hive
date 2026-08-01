---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: en
counterpart: ../ko/v0-9-full-release.md
title: "Aigent Hive 0.9.0 Full Release"
summary: "The authorized 0.9.0 release binds final artifacts, tag, GitHub Release, and npm publication to one protected main commit."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:123404518f674d04bc55b19726a172c28d0fd7e51b2f6d5c63ffbc1f55889a60"
links: [release-verification, test-distribution, version-policy]
reviewed_revision: "git:2f7acd20ba3c7d79e4cf98ed84c3a4807915d55f"
status: active
---

# Aigent Hive 0.9.0 Full Release

Authorized scope: plan exact `0.9.0` and push its baseline to remote `develop`.
Normal fast-forward pushes to `develop` remain allowed; deletion and non-fast-
forward updates are blocked. Final production `main` requires a pull request and
four release checks. Strict `staging` exists only when an approved plan needs it;
this flow does not. Publication binds one protected `main` commit to tag `v0.9.0`,
GitHub Release, five signed native artifacts, six npm packages, and installers.
Apple and Windows signing, external TUF authorization, protected approvals, and
public install and upgrade evidence remain mandatory. Other versions, force-push,
branch deletion, and credential custody are outside scope.
