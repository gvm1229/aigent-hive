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
  - "repo:.github/workflows/release-runtime.yml#sha256:398fa5b776385221e2e98762895c46ff92d1a5b17b8cd8f414347b40e9c5303f"
  - "repo:docs/archive/plans/foundations/phases/07-public-qualification.md#sha256:4340322bc0dfdc4029e7d5366ad40bfd0c4bd53f33b9b8ebc1e82f1a524cbf06"
links: [test-distribution]
reviewed_revision: "git:e37de7ff99fb235f673a4d3273deb54d6284999e"
status: active
---

# Linux musl qualification

GitHub run `30581894132`: x86_64·arm64 musl qualification 통과.
수용 범위: locked release build, ELF architecture·static linkage, exact package
layout, archive digest, installed binary 실행, isolated Antigravity install lifecycle.
도입 맥락: 사용자 요청 `0.8.0` 시험 배포 Linux 지원.
