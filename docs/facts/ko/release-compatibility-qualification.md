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
  - "repo:.github/workflows/release-runtime.yml#sha256:0b752edf707f676d71f5bee91da1fb61e5bdfe0de8321f71ade6011229d431a6"
  - "repo:docs/archive/plans/releases/0.9.5/release-0.9.5-stable-publication.md#sha256:70ed823701fa0ae8be728d97b8705846f0eaa50e6e8758425d439bfee4d1334c"
links: [historical-project-base-coverage, release-verification, test-lane-inventory]
reviewed_revision: "git:a520f8e5113c7fc02711eb5e5d8021605f7ee551"
status: active
---

# 출시 호환성 수용

- `0.9.5`: npm `latest`·GitHub Release `v0.9.5` 게시
- M2 MacBook Air 공개 설치: 격리 user setup·Codex install·validate·stable update check 성공
- native runtime: prior patch tag 기반 release-version 검사로 `fetch-depth: 0` 필요
- Windows x64 공개 npm binary identity 확인
- Windows 격리 Codex `0.148.0` setup·install·validate·stable update current 성공
- 보정 뒤 native runtime run `32118217691`: 다섯 대상 성공
