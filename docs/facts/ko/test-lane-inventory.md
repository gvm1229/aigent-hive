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
  - "repo:scripts/test-lanes.py#sha256:08d6ee2113e301f836a217733539f9e01b96f4c6569f4f71c4e02635fab0bfa8"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:5da00009c7146f04088445cfeda29641f7edc22757f9a9c9e37b5ea0a612bcf0"
  - "repo:tests/conformance/integration/test_connected_setup_lifecycle.py#sha256:316c4057978fb4b928618c41fb37fb596f9d8b8d9e6e4f08fe85cdfa8756ada0"
  - "repo:tests/conformance/lanes.toml#sha256:e489bbf237207fd643f36a4e95324c977de54368f87cc74b03646ee19549f693"
links: [release-verification, test-fault-isolation]
reviewed_revision: "git:3b4d6d23c679eec9e23f334dc60a2678b657345e"
status: active
---

# 시험 lane 대장

`tests/conformance/lanes.toml`: 모든 `test_*.py` 모듈의 documentation·security·contract·integration·release
중 하나 배정. `scripts/test-lanes.py`: 누락·중복 거부, 선택 lane 실행, 경과 시간 기록. 소비자 fixture:
ignored `tests/work/` root 사용. Phase 4: tracked test 인접 `tests/hive-phase4-*` 생성 중단.
