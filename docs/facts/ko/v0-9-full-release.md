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
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:9172a8fa815052211dac6f561775f47852f4fe86bd629cb02004bbf5e0e30acb"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:72924a588ff5b37e8ef76a19da3449096939599d5b3e1e2dc03ab44ec3281bd3"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:a78aed2efcf96d34ef020addc30ebdd70f035286"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

시험 prerelease: npm `test`·GitHub prerelease, stable `v0.9.0` 부재. `dc4466d`의 candidate
`31134306991`, publication `31135040224`으로 `v0.9.0-test.4` 22-asset prerelease 생성. 여섯
package `test=0.9.0-test.4`, `latest=0.8.0`; annotated prerelease tag `v0.9.0-test.4` 확인.
격리 설치 CLI label `AIgent Hive v0.9.0-test #4 · developer test build (released 2026-08-07)` 출력 확인.
