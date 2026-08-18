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
  - "repo:crates/hive-cli/src/main.rs#sha256:0d4ace682bb4bc6cd31a5a3fa4675e1b0a1f2bdb93f55d9456177be7e629cd9d"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b8bb2ace9f509cf8f48cf703971069e7ca73ada3704a8c7dc18adfd03a27e9ef"
  - "repo:docs/hive-install-guide.ko.html#sha256:31a2c507fb0b2d266c012ca62cfd91a69b9e6847deaf8eaa1a3abe455ea83d85"
  - "repo:docs/plans/active/multi-host-user-install.md#sha256:048a38d199eb35e838d0772e8162537708f0a006de50614992cd88be49bbb820"
links: [global-onboarding, supported-hosts]
reviewed_revision: "git:565b41f08d02db2308356f1cb5ed35d901337a4b"
status: active
---

# 복수 호스트 사용자 설치

`hive install`·사용자 범위 `hive update`: 기존 단일 `--host` 호환과
`--hosts codex,claude`·반복 `--host codex --host claude` 지원.
쉼표 주변 공백: 단일 argument 안에서 허용. Shell 입력 예시: `--hosts "codex, claude"`.
Host 순서: 요청 순서. Duplicate·empty·unknown 선택: mutation 전 거부.
복수 apply: 모든 host dry-run 뒤 순차 실행.
Aggregate JSON: host별 결과, 후속 실패 시 완료·실패 host와 유지된 변경 경로.
