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
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:9b55b1293372b76fd080e98f49d1307c26c5bdbc9c39100364f59ce2719d50a5"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:a7a53138de9f3be5e4627e8ac7781cb1ed7bbd968712d6d1bc040502186aca9d"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:a78aed2efcf96d34ef020addc30ebdd70f035286"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

시험 prerelease는 npm `test`·GitHub prerelease이며 stable `v0.9.0`은 아직 없다. 최신 공개
시험판은 `0.9.0-test.5`이고, 여섯 package의 `test=0.9.0-test.5`, `latest=0.8.0` 유지가 확인됐다.
commit `9e08a48`의 `0.9.0-test.6` 후보 `31254605322`는 5개 native
target·npm 묶음·GitHub attestation을 통과했다. 그러나 Trusted Publishing `31255061771`과 bootstrap
fallback `31255167232`은 모두 첫 scoped package에서 npm `404`로 중단했다. 따라서
`0.9.0-test.6` npm version·tag·GitHub prerelease 생성은 0건이다.
