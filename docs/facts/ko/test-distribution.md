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
  - "repo:docs/decisions/ADR-0013-0.8-release-scope.md#sha256:0a517849c6db000119e9c677b25304ce87a167b094473990dd9a1bb60ab609b6"
links: [global-onboarding, version-policy]
reviewed_revision: "git:cf992996d3076479bdfb433c4171eee046f571ae"
status: active
---

# 0.8.0 시험 배포

Exact `0.8.0`: npm exact version과 `test` tag의 install·update 시험 배포.
GitHub Release·release tag·npm `latest` 이동 0건.
Umbrella tarball의 rendered direct installer가 exact scoped platform package
digest를 검증한 뒤 native binary 설치.
