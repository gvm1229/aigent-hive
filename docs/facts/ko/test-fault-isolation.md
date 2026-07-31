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
  - "repo:crates/hive-render/src/lib.rs#sha256:0e951341e2019cae36ad1bca31e7b20660abe68efb99f301093ce5786e989640"
  - "repo:crates/hive-update/src/transaction.rs#sha256:d55b9b13726eb812ffdf0e605fe41a24a343157bd41ca175c6750aa6443154ec"
links: [test-distribution]
reviewed_revision: "git:ed553c9b397c2ce5c0586c28b8aa665bea842c0d"
status: active
---

# 시험 장애 주입 격리

Rust 단위 시험의 장애 주입: 해당 시험 스레드로 범위 제한.
효과: 병렬 갱신 시험의 장애 주입 오소비 차단.
격리된 CLI 하위 프로세스 적합성 시험을 위한 숫자형 프로세스 범위 호환 유지.
수용 기준: 구문 분석기 회귀 시험과 반복 병렬 `hive-update` 시험 전체 완료
도입 배경: 사용자 요청 `0.8.0` 시험 배포 검증.
