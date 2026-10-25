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
  - "repo:docs/guides/test-cleanup.md#sha256:12d06ecf62144cec1394e4cff8762ce8b7942211941077f1d232728182c0294f"
  - "repo:docs/guides/test-lanes.md#sha256:ac5e2863835c6c3605986ff71600d6b6a626676dc745a781cc2d0e18a28fa451"
  - "repo:scripts/test-lanes.py#sha256:5bc7694c5e1f399880069d16edbde37b85c741dadc5d6252892ebd5142cea8b1"
  - "repo:scripts/test_artifacts.py#sha256:576ee2abad8e8788b0ca22ca617e9fa048afef8321ba4dc3e8f150d7a5bd7978"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:aadd1bc3e37924cf0869cb520cbf4401583682c3b005d19a4f119d3b381b6524"
  - "repo:tests/conformance/integration/test_connected_setup_lifecycle.py#sha256:43ff866b01e2ff98234a12225e2278b1fd18752fa78ae3c34b91d1afefc6685d"
  - "repo:tests/conformance/lanes.toml#sha256:bc515db1c2bbb782a72a4d1b5ab659967f471b829b85d8b193248717d8532dd8"
links: [release-verification, test-fault-isolation]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# 시험 lane 대장

- phase 대신 목적별 Python package와 `tests/conformance/lanes.toml`, 재귀 발견한 모든 test_*.py의 documentation·security·contract·integration·release 배정
- 누락·중복 거절, 변경 경로 선택·모듈별 JSON 시간 기록, 안정성·과거 갱신 시험과 자료 보존
- daily: 기존 Git 근거·정확 경로 검토 뒤 정리, 미검토·만료·중복 제외 20GiB 초과 시 점검
- 시험 기본 CARGO_INCREMENTAL=0·명시 설정 보존, 실행 중·목록 변경·연결 삭제 거절
- 사용자 규약: 정리의 tests/ 새 문서·결과 커밋 금지, 감사 기록은 무시된 .agents/work/test-artifacts/cleanup/, 기존 시험 결과는 Git 보존
