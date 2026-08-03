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
  - "repo:docs/plans/active/release-0.9.0.md#sha256:06ea57eb932d8de296f9a910aceffe217733c9c243a7acc67d1676b58c2430d6"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:a5b7ebfb6ad70159fe33c4f94902e649eff0c504"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

기본 시험: `0.9.0-test`, npm `test`, GitHub prerelease; 추가 시험판: 필요 시에만
`0.9.0-test.N`. `6761f0b` candidate run `30771098518`: 5개 native target·npm umbrella
PASS. PR #16 `main` merge: workflow 등록. `develop` ref run `30789141992`:
`dist/...` Git remote parse failure, 첫 npm 게시 전 중단, npm·tag·GitHub Release mutation
0건. Commit `3782475`: 두 publish path `./$archive` 보정, regression·full pre-push PASS.
Retry `30808850724`: `release-publication` protected approval 대기. `deployment: false`:
approval·secret 유지, 신규 Deployment record 0건. 시험 publication의 `latest` 변경·정식
trigger 0건. 시험·정식 parity: report preview·export, `markdown|notion`, optional Discord guard.
