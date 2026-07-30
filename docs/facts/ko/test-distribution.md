---
schema_version: 1
pair_id: test-distribution
topic_slug: test-distribution
language: ko
counterpart: ../en/test-distribution.md
title: "0.8.0 시험 배포"
summary: "GitHub Release·npm latest 없는 npm test 전용 0.8.0."
tags: [distribution, release, test]
aliases: ["0.8.0 release scope"]
sources:
  - "repo:docs/decisions/ADR-0013-0.8-release-scope.md#sha256:2fb97b133d567155c0f333cbe7a401fc7473e849d88db2e2f9b897d7acecb39e"
links: [global-onboarding, version-policy]
reviewed_revision: "git:99f39edd08cc4b9d513f073d297bed05e2772c9d"
status: active
---

# 0.8.0 시험 배포

Exact `0.8.0`: npm exact version과 `test` tag의 install·update 시험 배포.
GitHub Release·release tag·npm `latest` 이동 0건.
Umbrella tarball의 rendered direct installer가 exact scoped platform package
digest를 검증한 뒤 native binary 설치. Candidate activation·publication은
명시적으로 선택된 protected branch가 필수.
