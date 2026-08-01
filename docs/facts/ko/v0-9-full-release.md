---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.0 정식 릴리스"
summary: "0.9.0 최종 artifact·tag·GitHub Release·npm publication을 하나의 protected main commit에 결합"
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:123404518f674d04bc55b19726a172c28d0fd7e51b2f6d5c63ffbc1f55889a60"
links: [release-verification, test-distribution, version-policy]
reviewed_revision: "git:2f7acd20ba3c7d79e4cf98ed84c3a4807915d55f"
status: active
---

# Aigent Hive 0.9.0 정식 릴리스

관리자 승인 범위: exact `0.9.0` 정식 릴리스 계획과 구현 기준선의 원격 `develop`
push. Publication 계보: protected `main` 단일 commit의 final candidate, annotated
`v0.9.0`, GitHub Release, signed native artifact 5개, npm package 6개, direct
installer. Branch authority: `develop` 일반 fast-forward push 허용, deletion·
non-fast-forward 차단; final production `main`은 PR과 release check 4개 필수.
엄격한 `staging`은 승인된 release plan이 요구할 때만 생성하며 현재 흐름에는
불필요. 필수 선행 증거: Apple·Windows signing, external TUF authorization,
protected approval, public install·upgrade. 범위 제외: 다른 version, force-push,
branch deletion, credential custody.
