---
schema_version: 1
pair_id: test-fault-isolation
topic_slug: test-fault-isolation
language: ko
counterpart: ../en/test-fault-isolation.md
title: "시험 장애 주입 격리"
summary: "실행 중 장애 주입을 이를 소유한 Rust 시험 스레드로 제한."
tags: [release, test, update]
aliases: ["장애 주입 범위"]
sources:
  - "repo:crates/hive-render/src/lib.rs#sha256:8c8388a9b849e36f2937cd971a1ebe16b2e598c5bd3f71ade1354b8797919a17"
  - "repo:crates/hive-update/src/transaction.rs#sha256:cdde5a6e6cb9f3ff193a74061899f2c21e87e56ee2ace126b9f9e73c1cea9436"
links: [test-distribution]
reviewed_revision: "git:19eda4d7ef87fe3122c14c455df07758c3dc6ff1"
status: active
---

# 시험 장애 주입 격리

Rust 단위 시험의 장애 주입: 해당 시험 스레드로 범위 제한.
효과: 병렬 갱신 시험의 장애 주입 오소비 차단.
격리된 CLI 하위 프로세스 적합성 시험을 위한 숫자형 프로세스 범위 호환 유지.
수용 기준: 구문 분석기 회귀 시험과 반복 병렬 `hive-update` 시험 전체 완료
도입 배경: 사용자 요청 `0.8.0` 시험 배포 검증.
