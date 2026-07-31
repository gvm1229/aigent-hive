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
  - "repo:.github/workflows/release-runtime.yml#sha256:89cc2b2c3b209e48e48fdd13b032c6c72eea612246ecf376d3e9d71f30702b63"
  - "repo:docs/plans/phases/07-public-qualification.md#sha256:379df42cc0d33872117fe1f484a24aa4fba06805f1e9dafb9b0e07098ee04f83"
links: [test-distribution]
reviewed_revision: "git:a7be86f2558442c2cec3596abe2f481dd91d268f"
status: active
---

# Linux musl Qualification

GitHub run `30581894132` qualified both `x86_64-unknown-linux-musl` and
`aarch64-unknown-linux-musl`. Acceptance covered locked release builds, ELF
architecture and static linkage, exact package layout, archive digest, installed
binary execution, and isolated Antigravity install lifecycle. Origin: the requested
Linux support for the `0.8.0` test distribution.
