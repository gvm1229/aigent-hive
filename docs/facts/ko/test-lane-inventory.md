---
schema_version: 1
pair_id: test-lane-inventory
topic_slug: test-lane-inventory
language: ko
counterpart: ../en/test-lane-inventory.md
title: "시험 lane 대장"
summary: "목적별 시험 package와 단일 실행 lane 대장의 안정성 회귀 보존."
tags: [release, test, verification]
aliases: ["conformance lanes", "test inventory"]
sources:
  - "repo:docs/guides/test-lanes.md#sha256:2d9ea96838ebef0f85ad3bdc2549163fe8b01fcfb5d206ce2bef4a7be7763ee6"
  - "repo:scripts/test-lanes.py#sha256:08d6ee2113e301f836a217733539f9e01b96f4c6569f4f71c4e02635fab0bfa8"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:c77febdf50b689937897ea1848ae0f38468d14843dbeda5486678eb523447902"
  - "repo:tests/conformance/integration/test_connected_setup_lifecycle.py#sha256:f74ae9ecf4d442e4171b4f0b28bb4d2a7ad75167858d8cba436e9710021e12ab"
  - "repo:tests/conformance/lanes.toml#sha256:367c50ac018f90df25cb75d3a2daebf254eab52eeaac9f8d407373b0e83a099a"
links: [release-verification, test-fault-isolation]
reviewed_revision: "git:571467bb776b86bed509a06cdb6744434b067993"
status: active
---

# 시험 lane 대장

Python 시험·fixture: phase 디렉터리 대신 목적별 package 사용.
`tests/conformance/lanes.toml`: 재귀 발견한 모든 `test_*.py` 모듈의 documentation·security·
contract·integration·release 중 하나 배정. 실행기: 누락·중복 거부, 변경 경로 기반 lane 선택,
module 시간 JSON 제공. 재편 중 안정성·과거 upgrade 시험과 fixture 삭제 `0건`.
