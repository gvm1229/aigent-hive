---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.0 시험·정식 릴리스"
summary: "0.9.0은 독립 bare 시험 채널의 수용 뒤 별도 승인된 정식 publication 사용."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:d6bcfc0ec1e77c1f76cb8e24e7686457311099026f8af7cd191ad568d351e1ec"
links: [release-verification, test-distribution, version-policy]
reviewed_revision: "git:5e09d0ff23e841381c22bac24e707dbc6402dae4"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

기본 시험 identity: package `0.9.0-test`, npm `test`, GitHub prerelease. 추가 시험판:
필요 시에만 `0.9.0-test.N`. 시험 publication의 `latest` 변경·정식 publication trigger
0건. 시험·정식 artifact의 기능·기본값·진단 계약 동일. 소비자 공통 문제 보고:
명시적 preview·export, 자동 upload 0건. 정식 `0.9.0|latest`: 시험 수용 뒤 별도
protected `main` publication. 정식 선행 증거: Apple·Windows signing, external TUF,
protected approval, public install·upgrade.
