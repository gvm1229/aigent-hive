---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: en
counterpart: ../ko/v0-9-full-release.md
title: "Aigent Hive 0.9.0 Test and Stable Release"
summary: "test.13 completed Windows preserving-reinstall acceptance; unattended default-profile acceptance and the measured test-lane inventory are complete before the main stable candidate."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:5c00353f7683e586ada9ccfec9e80dd7504d2f464d88309ea8d9786f916219d5"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:8ad5eecbc7f16667965b4e97a5e51c4c50e00f50e5677bddbf1b9a34fc11f943"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:3e960b5185f637d7606eb01126d2543519138608"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Stable `v0.9.0` remains absent. Candidate `31403054797` and OIDC publication `31404195752`
released `0.9.0-test.13` from `03a16676`; all six npm packages have `test=0.9.0-test.13` and
`latest=0.8.0`. Windows preserving uninstall, clean reinstall, `dry-run → apply → validate`,
install validation, new-session `hive` discovery, and Discord delivery are accepted.

The product-owned expedited default now passes clean install and preserving reinstall without
contributor preferences or setup dialogue; reinstall restores its Hive-owned user projection.
Every Python conformance module has one measured lane; the CI matrix runs those lanes separately.
The next gate is the non-force `develop → main` merge and exact-main stable candidate.
