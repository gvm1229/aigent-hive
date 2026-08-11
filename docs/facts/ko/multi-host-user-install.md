---
schema_version: 1
pair_id: multi-host-user-install
topic_slug: multi-host-user-install
language: ko
counterpart: ../en/multi-host-user-install.md
title: "복수 호스트 사용자 설치"
summary: "Hive 사용자 설치·update의 CSV·반복 host 선택과 aggregate 결과 지원."
tags: [installation, multi-host, user-setup]
aliases: ["복수 호스트 설치", "여러 호스트 설치"]
sources:
  - "repo:crates/hive-cli/src/main.rs#sha256:f8ea20501bfcc0226a8f720c7e18c5b772389aa423d3796ed8c440d1759bc671"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:6e7acffee805aea9462b9f350be5d99b9220f719894a19d384e4a5f4756822d9"
  - "repo:docs/hive-install-guide.ko.html#sha256:1e7113e8769a0f2cc6e0c1b8848144ff51be8a46b0605e48454a7359fed24e93"
  - "repo:docs/plans/active/multi-host-user-install.md#sha256:1737d5e9f4960e21f112968754cab385d5976a6114dcf1a1d0da9ae19ed2d6fd"
links: [global-onboarding, supported-hosts]
reviewed_revision: "git:4f977874ee41c19ac76a89ce61a9a4ac05ee780a"
status: active
---

# 복수 호스트 사용자 설치

`hive install`·사용자 범위 `hive update`: 기존 단일 `--host` 호환과
`--hosts codex,claude`·반복 `--host codex --host claude` 지원.
Host 순서: 요청 순서. Duplicate·empty·unknown 선택: mutation 전 거부.
복수 apply: 모든 host dry-run 뒤 순차 실행.
Aggregate JSON: host별 결과, 후속 실패 시 완료·실패 host와 유지된 변경 경로.
