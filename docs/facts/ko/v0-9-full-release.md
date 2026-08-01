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
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:8e430be6ee5b2497afd32eaf009aab56698b1ae7bcbef5988358e9c3e3436e47"
links: [release-verification, test-distribution, version-policy]
reviewed_revision: "git:d0747ee7e1851b9edfa2066214e948d75e895ebd"
status: active
---

# Aigent Hive 0.9.0 정식 릴리스

관리자 승인 범위: exact `0.9.0` 정식 릴리스 계획과 구현 기준선의 원격 `develop`
push. Publication 계보: protected `main` 단일 commit의 final candidate, annotated
`v0.9.0`, GitHub Release, signed native artifact 5개, npm package 6개, direct
installer. 필수 선행 증거: Apple·Windows signing, external TUF authorization,
protected approval, public install·upgrade. 범위 제외: 다른 version, force-push,
branch deletion, credential custody.
