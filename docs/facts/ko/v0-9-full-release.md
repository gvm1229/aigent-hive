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
  - "repo:docs/plans/active/release-0.9.0.md#sha256:979c5cc733a3cc7d2397fcae1ce689036558f9dd297751e3562c9b6498500d52"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:0a2fb65ae90b93fb111fd75acff42e917692b69e"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

기본 시험은 `0.9.0-test`·npm `test`·GitHub prerelease이며 `.N`은 선택형. `6761f0b`의
candidate `30771098518`과 maintainer recovery로 `v0.9.0-test` 22-asset prerelease 생성.
App token candidate `31042797141`의 `dd0224a`, publication `31043631056`으로
`v0.9.0-test.1` 22-asset prerelease 생성. 여섯 package `test=0.9.0-test.1`,
`latest=0.8.0`; stable `v0.9.0`·npm `0.9.0` 부재.
