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
  - "repo:.github/workflows/release-runtime.yml#sha256:e02b4cfeaf85ed248bd09113bb208e6e3c72a083cd510309c5d3f718c90d3fa8"
  - "repo:docs/plans/active/release-0.9.5-stable-publication.md#sha256:7681da81e7ae900184cbaaaffd51763f547e417d959b329f1e1e42f167867475"
links: [historical-project-base-coverage, release-verification, test-lane-inventory]
reviewed_revision: "git:a49b4c9520a9099f41da1a70ea543eaf445e1053"
status: active
---

# 출시 호환성 수용

- `0.9.5`: npm `latest`·GitHub Release `v0.9.5` 게시
- M2 MacBook Air 공개 설치: 격리 user setup·Codex install·validate·stable update check 성공
- native runtime: prior patch tag 기반 release-version 검사로 `fetch-depth: 0` 필요
- 남은 별도 증거: Windows x64 public stable installer 수용, 보정 뒤 native runtime 다섯 대상 재실행
