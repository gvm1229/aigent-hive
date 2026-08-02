---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.0 시험·정식 릴리스"
summary: "0.9.0은 독립 게시 가능한 bare 시험 채널의 수용 뒤 별도 승인된 정식 publication 사용."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:9172a8fa815052211dac6f561775f47852f4fe86bd629cb02004bbf5e0e30acb"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:c15f515371704df7137aed30358690c127df14e99002d4be26b3542fa4e286cf"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:e48af805488a328ae910e6c76e3e08d5a0fa2d33"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

기본 시험: `0.9.0-test`, npm `test`, GitHub prerelease. 추가 시험판: 필요 시에만
`0.9.0-test.N`. `6761f0b` candidate run `30771098518`: 5개 native target·npm umbrella
PASS. Public 시험 publication: default-branch workflow 등록 전까지 차단, 실패 dispatch의
npm·tag·GitHub Release mutation 0건. 등록 전용 `main` PR·review 뒤 protected publication.
시험 publication의 `latest` 변경·정식 trigger 0건. 시험·정식 artifact 동일:
feature·default·diagnostic, report preview·export, `markdown|notion`, optional Discord
guard. 정식: 시험 수용 뒤 별도 protected `main` publication.
