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
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:8e430be6ee5b2497afd32eaf009aab56698b1ae7bcbef5988358e9c3e3436e47"
links: [release-verification, test-distribution, version-policy]
reviewed_revision: "git:d0747ee7e1851b9edfa2066214e948d75e895ebd"
status: active
---

# Aigent Hive 0.9.0 Full Release

The maintainer authorized planning the exact `0.9.0` full release and pushing the
implementation baseline to remote `develop`. Publication must rebuild the final
candidate from one protected `main` commit and bind that commit to annotated tag
`v0.9.0`, the GitHub Release, five signed native artifacts, six npm packages, and
direct installers. Apple and Windows signing, external TUF authorization,
protected approvals, and public install and upgrade evidence remain mandatory.
The request grants no authority for another version, force-push, branch deletion,
or credential custody.
