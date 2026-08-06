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
  - "repo:docs/plans/active/release-0.9.0.md#sha256:eb18df03b5a407f3fb3a405a9af0dd146ff653d92dbc5ba6528f08198efedc7c"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:6980e8b38c08a9ebe483a4ffa7937f70999d63a5"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

시험 prerelease: npm `test`·GitHub prerelease, stable `v0.9.0` 부재. `6980e8b`의 candidate
`31082481203`, publication `31083602464`으로 `v0.9.0-test.2` 22-asset prerelease 생성. 여섯
package `test=0.9.0-test.2`, `latest=0.8.0`; 격리 install의 developer-test build label·`2026-08-06`
출력 확인. Trusted publishing `31083140684`의 scoped-package `404` 2회, mutation 0건;
existing registry-auth fallback publication 완료.
