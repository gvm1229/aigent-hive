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
  - "repo:docs/plans/active/release-0.9.0.md#sha256:06f4243bc56e0a75525bae3c838bf8599e26d9e143fc8f75ff332134f42dd468"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:0a2fb65ae90b93fb111fd75acff42e917692b69e"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

기본 시험은 `0.9.0-test`·npm `test`·GitHub prerelease이고 `.N`은 선택이다. `6761f0b`
candidate run `30771098518`은 5개 native target·npm umbrella PASS. reviewer 없는 bootstrap
run `30890841117`은 여섯 package를 게시하고 `test=0.9.0-test`, `latest=0.8.0`을 확인했다.
마지막 tag/Release는 workflow-tag 권한 부족으로 실패했다. authenticated maintainer recovery가
같은 candidate annotated `v0.9.0-test`와 22 asset prerelease를 생성했다. stable `v0.9.0`,
npm `0.9.0`, `latest` 변경은 0건이다. 이후 완전 자동 finalization은 contents·workflows write의
repo-scoped GitHub credential을 별도 authority로 보관해야 하며 현재 credential은 복제하지 않는다.
