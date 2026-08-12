---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.x 시험·정식 릴리스"
summary: "v0.9.2: 완료된 usage guard와 공개 문서 전수 최신화, v0.9.3: 후속 명시적 승인 필요"
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.2 scope", "0.9.3 scope", "0.9.x release plan", "full release"]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:903c4fd819d0d09afdbc379ac874a22d592274b495aab6de82ee15166381bcbb"
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:0d2220c5b07d579fd7c54d958380b2482c1e44c9b738651840d119c79692b5be"
  - "repo:docs/guides/release-update.md#sha256:785e83d497c4f39ea683ac280adf8e071b27fda02b19c4c086573782a70bcadb"
  - "repo:docs/plans/active/release-0.9.2-test-qualification.md#sha256:2c2d6a00e695dc549649d5eb0c8416986dc5962e88c15a2f4836ee715eee821f"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:3f6e8607d62a8905abb68aabc599f88a573c08f1"
status: active
---

# Aigent Hive 0.9.x 시험·정식 릴리스

Stable `v0.9.1`: exact source `1e5e7b3`에서 게시 완료. `0.9.2`: `2cec037`까지 완료된 설치
usage guard 정본 전환과 release-only metadata·qualification 범위. `c777da1` 이후 Native
orchestration·custom subagent 작업: 별도 branch의 `0.9.3` 이관. Stable publication:
numbered public test 수용 뒤 최종 배포 전용. `0.9.2` gate: 모든 공개 README·설치 안내·HTML·
npm README·plugin metadata·문서 색인·명령·version 예시 최신화. 기존 `develop` history 보존.
`0.9.3`: QA contributor 추가 지시와 유지보수자의 후속 명시적 승인 전 동결.
