---
schema_version: 1
pair_id: version-policy
topic_slug: version-policy
language: en
counterpart: ../ko/version-policy.md
title: "Version Policy"
summary: "Version numbering and perpetual compatibility from the first public stable release, 0.9.1."
tags: [release, semver, version]
aliases: ["Version lifecycle"]
sources:
  - "repo:docs/decisions/ADR-0006-version-lifecycle.md#sha256:c49c0f5cba2370bf031d454ed1e5fbc0fd70fa07793f55e14eeafe0d45e8f066"
links: [release-verification, test-distribution]
reviewed_revision: "git:c6d9362af3b489596fafe933e816fb2126bff83d"
status: active
---

# Version Policy

A backward-compatible feature requires the exact next minor version and a compatible
fix the exact next patch. A major target requires the user's exact version and separate confirmation.

The first public stable release is 0.9.1. Earlier versions are outside backward-compatibility
and historical-recall scope; this does not authorize deleting historical artifacts.
Every version from 0.9.1 through the latest must remain backward compatible, now and for
all future releases, not merely the immediate predecessor. A version increase cannot waive
this obligation. This is the maintainer's requirement, not proof that all compatibility tests passed.
