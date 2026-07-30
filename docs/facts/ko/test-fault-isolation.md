---
schema_version: 1
pair_id: test-fault-isolation
topic_slug: test-fault-isolation
language: ko
counterpart: ../en/test-fault-isolation.md
title: "시험 fault 격리"
summary: "In-process activation fault를 소유 Rust 시험 thread로 제한."
tags: [release, test, update]
aliases: ["activation fault scope"]
sources:
  - "repo:crates/hive-render/src/lib.rs#sha256:46d4bbf77befbc60d21f3787299beeb0f2e3d4acf102fd772c751936c26aa1c4"
  - "repo:crates/hive-update/src/transaction.rs#sha256:d55b9b13726eb812ffdf0e605fe41a24a343157bd41ca175c6750aa6443154ec"
links: [test-distribution]
reviewed_revision: "git:235d5e39a36b4c9d395a65ee4fa7a7ed52515768"
status: active
---

# 시험 fault 격리

Rust unit test의 injected activation failure: 소유 test thread로 범위 제한.
효과: 병렬 update 시험의 fault 오소비 차단.
격리 CLI subprocess conformance용 numeric process scope 호환 유지.
수용 기준: parser 회귀 시험과 반복 병렬 `hive-update` suite 통과.
도입 맥락: 사용자 요청 `0.8.0` 시험 배포 qualification.
