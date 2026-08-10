---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: en
counterpart: ../ko/v0-9-full-release.md
title: "Aigent Hive 0.9.0 Test and Stable Release"
summary: "test.12 completed Windows acceptance; unattended default-profile acceptance and the measured test-lane inventory are complete before the next numbered candidate."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:9a4853952266b6a234ecf88bda90eebbf16148b8e529aa7739c9321c45866b91"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:036a7ea7282cf0ed6ffe0bef403331b73249cc566b2e381f8f43efc13e4097e3"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:3e960b5185f637d7606eb01126d2543519138608"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Stable `v0.9.0` remains absent. Candidate `31391084832` and OIDC publication `31392103115`
released `0.9.0-test.12` from `c98add0`; all six npm packages have `test=0.9.0-test.12` and
`latest=0.8.0`. Windows preserving uninstall, clean reinstall, `dry-run → apply → validate`,
install validation, new-session `hive` discovery, and Discord delivery are accepted.

The product-owned expedited default now passes clean install and preserving reinstall without
contributor preferences or setup dialogue; reinstall restores its Hive-owned user projection.
Every Python conformance module has one measured lane; the CI matrix runs those lanes separately.
The next gate is a numbered candidate and public test acceptance before `main`.
