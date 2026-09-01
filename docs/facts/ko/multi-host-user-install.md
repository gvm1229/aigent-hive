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
  - "repo:crates/hive-cli/src/main.rs#sha256:024500782daa35d5ab3a6df26a443bf0e4c0653a2a2c19caaa2f1b2a7836cdb6"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:7a5c873834ba9a77e6efdedc60a5eed953fa40102dfcf88c084db5b591f465c3"
  - "repo:docs/archive/plans/foundations/multi-host-user-install.md#sha256:048a38d199eb35e838d0772e8162537708f0a006de50614992cd88be49bbb820"
  - "repo:docs/hive-install-guide.ko.html#sha256:fb6dbc2e5752fe9693c7ac791841b2e2f9995ef03a2d5c3ef4ce07b6f068800c"
links: [global-onboarding, supported-hosts]
reviewed_revision: "git:f91816a46d44d57929cb0b580ca32ff4caa95053"
status: active
---

# 복수 호스트 사용자 설치

`hive install`·사용자 범위 `hive update`: 기존 단일 `--host` 호환과
`--hosts codex,claude`·반복 `--host codex --host claude` 지원.
쉼표 주변 공백: 단일 argument 안에서 허용. Shell 입력 예시: `--hosts "codex, claude"`.
Host 순서: 요청 순서. Duplicate·empty·unknown 선택: mutation 전 거부.
복수 apply: 모든 host dry-run 뒤 순차 실행.
Aggregate JSON: host별 결과, 후속 실패 시 완료·실패 host와 유지된 변경 경로.
