---
schema_version: 1
pair_id: release-compatibility-qualification
topic_slug: release-compatibility-qualification
language: ko
counterpart: ../en/release-compatibility-qualification.md
title: "출시 호환성 수용"
summary: "게시된 0.9.5의 전체 이력 native runtime 검증과 호환성 수용 근거"
tags: [compatibility, migration, release, testing]
aliases: ["호환성 matrix gate"]
sources:
  - "repo:.github/workflows/release-runtime.yml#sha256:06e8657e24d89fd2b28d87208fc15eb76d4b60357c1dd9d3c9c7c315b563d350"
  - "repo:docs/plans/active/release-0.9.5-stable-publication.md#sha256:e00beda4bab8467a5fa667fdc1f2799403d216398e29597f67f937bf94d46e95"
links: [historical-project-base-coverage, release-verification, test-lane-inventory]
reviewed_revision: "git:a49b4c9520a9099f41da1a70ea543eaf445e1053"
status: active
---

# 출시 호환성 수용

- `0.9.5`: npm `latest`·GitHub Release `v0.9.5` 게시
- M2 MacBook Air 공개 설치: 격리 user setup·Codex install·validate·stable update check 성공
- native runtime: prior patch tag 기반 release-version 검사로 `fetch-depth: 0` 필요
- 남은 별도 증거: Windows x64 public stable installer 수용, 보정 뒤 native runtime 다섯 대상 재실행
