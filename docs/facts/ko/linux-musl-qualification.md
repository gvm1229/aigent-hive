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
  - "repo:docs/plans/phases/07-public-qualification.md#sha256:34c8176ccfceccab6bfd33ce0318f1ed62c6ed44ac04bfd6f60f67bc653ba99f"
links: [test-distribution]
reviewed_revision: "git:e37de7ff99fb235f673a4d3273deb54d6284999e"
status: active
---

# Linux musl qualification

GitHub run `30581894132`: x86_64·arm64 musl qualification 통과.
수용 범위: locked release build, ELF architecture·static linkage, exact package
layout, archive digest, installed binary 실행, isolated Antigravity install lifecycle.
도입 맥락: 사용자 요청 `0.8.0` 시험 배포 Linux 지원.
