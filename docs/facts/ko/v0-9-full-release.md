---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.0 시험·정식 릴리스"
summary: "0.9.0 시험 prerelease의 protected 독립 채널과 별도 승인 정식 publication 계약."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:f534f4713c0a95b9a5e7ad63eed1470cd4cfd720adb37ecaea85f0a5dfad5009"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:eb57729c43e676c42fcb133b60b0efc5d17f4400805758447728fad2b4de8027"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:a78aed2efcf96d34ef020addc30ebdd70f035286"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

시험 prerelease는 npm `test`·GitHub prerelease이며 stable `v0.9.0`은 아직 없다. 최신 공개
시험판은 `0.9.0-test.5`이고 `latest=0.8.0`은 유지됐다. commit `9e08a48`의
`0.9.0-test.6` 후보 `31254605322`는 모든 native target·npm 묶음·attestation을 만들었지만,
두 publication 시도는 첫 scoped package의 npm `404`에서 중단됐다. 별도 token fallback workflow 대신
channel을 묶는 단일 `release-publish.yml`을 사용한다.
