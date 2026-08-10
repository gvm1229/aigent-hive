---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: en
counterpart: ../ko/v0-9-full-release.md
title: "Aigent Hive 0.9.0 Test and Stable Release"
summary: "The 0.9.0 stable release remains blocked: test.9 published successfully, but a preserved test.8 Codex marketplace failure prevents its Windows user-setup dry-run."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:6f3521bbf939c70b51f3ebcb31c3019e174b558f19b2658e0d9cfb563bed02e0"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:72e809923518c17689bf12ce990f87cba3ab1eaa28770b08b817afc6f20a01ab"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Stable `v0.9.0` remains absent. Candidate `31367482147` and OIDC publication `31368361218`
released `0.9.0-test.9` from `f88b0e5`; all six npm packages have `test=0.9.0-test.9` and
`latest=0.8.0`, with an annotated GitHub prerelease. Its isolated Windows npm install reported
test build #9. The saved user answers passed no host mutation, but dry-run stopped because the
preserved test.8 transaction left a Codex marketplace entry whose root lacks a manifest.

`REL9-011` remains open pending Hive-managed recovery or clean user host state, then clean-install
and fresh-session acceptance on the maintainer's actual Windows 11 machine.
