---
schema_version: 1
pair_id: linux-musl-qualification
topic_slug: linux-musl-qualification
language: ko
counterpart: ../en/linux-musl-qualification.md
title: "Linux musl qualification"
summary: "Linux x86_64·arm64 musl native runtime qualification 통과."
tags: [linux, release, test]
aliases: ["P7-043"]
sources:
  - "repo:.github/workflows/release-runtime.yml#sha256:89cc2b2c3b209e48e48fdd13b032c6c72eea612246ecf376d3e9d71f30702b63"
  - "repo:docs/plans/phases/07-public-qualification.md#sha256:639cfdc5ba7e9ff34e2fc139fdacfcffdbcf35617db1ec4ada074fdc3a7b6e07"
links: [test-distribution]
reviewed_revision: "git:16279760c12914a1ae0660da9d154c6884a19a92"
status: active
---

# Linux musl qualification

GitHub run `30581894132`: `x86_64-unknown-linux-musl`·
`aarch64-unknown-linux-musl` qualification 통과.
수용 범위: locked release build, ELF architecture·static linkage, exact package
layout, archive digest, installed binary 실행, isolated Antigravity install lifecycle.
도입 맥락: 사용자 요청 `0.8.0` 시험 배포 Linux 지원.
