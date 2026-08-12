---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.x 시험·정식 릴리스"
summary: "v0.9.2: 완료된 usage guard 작업 출시, Native·custom subagent 작업은 v0.9.3 이관"
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.2 scope", "0.9.3 scope", "0.9.x release plan", "full release"]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:903c4fd819d0d09afdbc379ac874a22d592274b495aab6de82ee15166381bcbb"
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:e3b94d5ec0cd5de8540b9475eb897afa05af3c1094f4e4393d913645352b4846"
  - "repo:docs/guides/release-update.md#sha256:785e83d497c4f39ea683ac280adf8e071b27fda02b19c4c086573782a70bcadb"
  - "repo:docs/plans/active/release-0.9.2-test-qualification.md#sha256:fb30685362ade4cb001612984bf6787e0346edac3aea5457c69b1a75042a6f5b"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:8f9ce5b241a75e153af8ec05479a18c24323e944"
status: active
---

# Aigent Hive 0.9.x 시험·정식 릴리스

Stable `v0.9.1`: exact source `1e5e7b3`에서 게시 완료. `0.9.2`: `2cec037`까지 완료된 설치
usage guard 정본 전환과 release-only metadata·qualification 범위. `c777da1` 이후 Native
orchestration·custom subagent 작업: 별도 branch의 `0.9.3` 이관. Stable publication:
numbered public test 수용 뒤 최종 배포 전용. 기존 `develop` history 보존, reset·대량 revert·
force-push 없음.
