---
schema_version: 1
pair_id: test-lane-inventory
topic_slug: test-lane-inventory
language: ko
counterpart: ../en/test-lane-inventory.md
title: "시험 lane 대장"
summary: "단일 실행 manifest: 모든 Python 적합성 모듈의 하나의 이름 있는 출시 lane 배정."
tags: [release, test, verification]
aliases: ["conformance lanes", "test inventory"]
sources:
  - "repo:scripts/test-lanes.py#sha256:a5733a0e84b484c06f89b7a5f55d09053de3153a38e444bbf0188ff4c319fa4d"
  - "repo:tests/conformance/lanes.toml#sha256:a8eb72c346f1e165e654ea6e02b02182659b2ebf296fb4d885792dc020f5bfd6"
  - "repo:tests/conformance/test_connected_setup_lifecycle.py#sha256:56f9e6c055d0cf5487da805838e2ce09afae79fa5c172ec11832cd5ba601d7fd"
  - "repo:tests/conformance/test_phase4_contracts.py#sha256:931a18a69a2f065109133c25ad954e8214f4635ee2685f412343354a8f34e396"
links: [release-verification, test-fault-isolation]
reviewed_revision: "git:3b4d6d23c679eec9e23f334dc60a2678b657345e"
status: active
---

# 시험 lane 대장

`tests/conformance/lanes.toml`: 모든 `test_*.py` 모듈의 documentation·security·contract·integration·release
중 하나 배정. `scripts/test-lanes.py`: 누락·중복 거부, 선택 lane 실행, 경과 시간 기록. 소비자 fixture:
ignored `tests/work/` root 사용. Phase 4: tracked test 인접 `tests/hive-phase4-*` 생성 중단.
