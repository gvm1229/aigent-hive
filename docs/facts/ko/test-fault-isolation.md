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
  - "repo:crates/hive-render/src/activation.rs#sha256:33b2d062ff0344069aff6fe9bf682f3fd1f69043903a3a5755649df729d9e18f"
  - "repo:crates/hive-render/src/lib.rs#sha256:0ab91f6389a28a312d90f0aa6ccd5b783da55b2bc21227d533448b6426e548d7"
  - "repo:crates/hive-update/src/transaction.rs#sha256:9b6b1f0d01bd69c287783f4d4089e3a580d848a7ff9b97cd13ee3445540edca4"
links: [test-distribution]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# 시험 장애 주입 격리

Rust 단위 시험의 장애 주입: debug·release 빌드 모두 해당 시험 스레드로 범위 제한.
효과: 병렬 갱신 시험의 장애 주입 오소비 차단.
격리된 CLI 하위 프로세스 적합성 시험을 위한 숫자형 프로세스 범위 호환 유지.
수용 기준: 구문 분석기 회귀 시험과 반복 병렬 `hive-update` 시험 전체 완료
도입 배경: 사용자 요청 `0.8.0` 시험 배포 검증.
