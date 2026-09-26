---
schema_version: 1
pair_id: v0-11-stable-release
topic_slug: v0-11-stable-release
language: en
counterpart: ../ko/v0-11-stable-release.md
title: "0.11.0 Stable Release"
summary: "Accepted public test.9 was promoted from one exact main commit to the 0.11.0 stable channel."
tags: [release, stable, v0-11]
aliases: ["0.11.0 stable"]
sources:
  - "repo:docs/research/0.11.0-stable-release-2026-09-27.md#sha256:bd5ee6d27564a184e7ad164b22510bb55477ed0aa100d535fdb3e4dd313dab68"
links: [release-verification, stable-public-documentation]
reviewed_revision: "git:ed6d8f0aee79b73bb83b4e9a057be85af304c1f0"
status: active
---

# 0.11.0 Stable Release

Accepted public test.9 qualified the shipped product bytes on Windows, macOS, and Linux. PR #64 merged develop into main. Candidate 36269801223 built five native targets and six npm packages from main commit ed6d8f0a. The first publication left the npm packages published but failed a tag propagation check. Recovery 36271127837 verified the same bytes and completed the normal GitHub release and Discord steps. Six npm packages have exact 0.11.0 versions and latest tags. The official v0.11.0 tag resolves to ed6d8f0a. No stable-channel regression installation was used.
