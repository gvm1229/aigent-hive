---
schema_version: 1
pair_id: test-fault-isolation
topic_slug: test-fault-isolation
language: ko
counterpart: ../en/test-fault-isolation.md
title: "시험 장애 주입 격리"
summary: "debug·release 시험 빌드에서 실행 중 장애 주입을 이를 소유한 Rust 시험 스레드로 제한."
tags: [release, test, update]
aliases: ["장애 주입 범위"]
sources:
  - "repo:crates/hive-render/src/activation.rs#sha256:17682ee8e6b20f3ba51ea8e6aae3aeb3df546aae0e7059070b3107df85ebea59"
  - "repo:crates/hive-render/src/lib.rs#sha256:51ce6acb616c554de9ae60e90259c815b30ddf33608ffdec02c07d966f2b089c"
  - "repo:crates/hive-update/src/transaction.rs#sha256:dd52b7530d83488b92fc7f7d2a93889ad6c3c7f28bd68d31403da107e38e656d"
links: [test-distribution]
reviewed_revision: "git:be1a9fc625b4bdb79597be8c3c081381d23e23d3"
status: active
---

# 시험 장애 주입 격리

Rust 단위 시험의 장애 주입: debug·release 빌드 모두 해당 시험 스레드로 범위 제한.
효과: 병렬 갱신 시험의 장애 주입 오소비 차단.
격리된 CLI 하위 프로세스 적합성 시험을 위한 숫자형 프로세스 범위 호환 유지.
수용 기준: 구문 분석기 회귀 시험과 반복 병렬 `hive-update` 시험 전체 완료
도입 배경: 사용자 요청 `0.8.0` 시험 배포 검증.
