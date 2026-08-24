---
schema_version: 1
pair_id: linux-musl-qualification
topic_slug: linux-musl-qualification
language: en
counterpart: ../ko/linux-musl-qualification.md
title: "Linux musl Qualification"
summary: "Linux x86_64 and arm64 musl native runtime qualification passed."
tags: [linux, release, test]
aliases: ["P7-043"]
sources:
  - "repo:.github/workflows/release-runtime.yml#sha256:77e6364e1a86fc768ca99fd8b6b36ac9c144a46f39e86819631ffd149985cc48"
  - "repo:docs/archive/plans/foundations/phases/07-public-qualification.md#sha256:4340322bc0dfdc4029e7d5366ad40bfd0c4bd53f33b9b8ebc1e82f1a524cbf06"
links: [test-distribution]
reviewed_revision: "git:e37de7ff99fb235f673a4d3273deb54d6284999e"
status: active
---

# Linux musl Qualification

GitHub run `30581894132` qualified both x86_64 and arm64 musl targets. Acceptance
covered locked release builds, ELF
architecture and static linkage, exact package layout, archive digest, installed
binary execution, and isolated Antigravity install lifecycle. Origin: the requested
Linux support for the `0.8.0` test distribution.
