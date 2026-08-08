---
schema_version: 1
pair_id: windows-namespace-gate-timeout
topic_slug: windows-namespace-gate-timeout
language: ko
counterpart: ../en/windows-namespace-gate-timeout.md
title: "Windows namespace gate timeout"
summary: "부하가 있는 Windows CI setup에 최대 30초를 허용하는 foreign-namespace gate."
tags: [ci, test, windows]
aliases: ["Windows setup timeout"]
sources:
  - "repo:tests/conformance/test_phase1_stage2_gates.py#sha256:abe47367fbfef26d7159cdafa771952a0bc8b43d3db4af8e088fa2268256d53b"
links: [test-fault-isolation]
reviewed_revision: "git:5f50bcde8a96782e95023486232992cc41b9abc1"
status: active
---

# Windows namespace gate timeout

POSIX foreign-namespace setup gate: 2초 FIFO detection 유지.
Windows test: FIFO probe 부재로 loaded CI setup에 30초 허용, exact before/after
namespace snapshot을 읽기·쓰기 판정 기준으로 사용.
수용 기준: Windows targeted 반복 실행과 전체 gate module PASS.
목적: 정상적인 느린 setup의 forbidden namespace read 오판 방지.
