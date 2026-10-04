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
  - "repo:docs/guides/test-cleanup.md#sha256:a4a41cadeebab8c9e685c7c5090fdc11b7a80d224125cd8b67f3411cb0623ac2"
  - "repo:docs/guides/test-lanes.md#sha256:ac5e2863835c6c3605986ff71600d6b6a626676dc745a781cc2d0e18a28fa451"
  - "repo:scripts/test-lanes.py#sha256:5bc7694c5e1f399880069d16edbde37b85c741dadc5d6252892ebd5142cea8b1"
  - "repo:scripts/test_artifacts.py#sha256:4d0aa57e8e0b0f80741a93f75584d9459513a8d6f6131f7d5f1aca1aa131dbd1"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:df8aa9994a9fa02a4ee782567f646f664d7414ca244aa679e49498a7832b041f"
  - "repo:tests/conformance/integration/test_connected_setup_lifecycle.py#sha256:81d38458c1fb4e2b0ad406bac350d06b5df34b57de31d729509f726402e9b319"
  - "repo:tests/conformance/lanes.toml#sha256:28e9d1ab7c0edb9325c4f923708982f21be0f963395f8cc27ca67df38abd065a"
links: [release-verification, test-fault-isolation]
reviewed_revision: "git:6c7b1c159e7b3ec553529818674f02c09151ff7b"
status: active
---

# 시험 lane 대장

- 목적별 Python package와 `tests/conformance/lanes.toml`의 단일 대장
- documentation·security·contract·integration·release 배정, 누락·중복 거절
- 변경 경로별 선택·모듈별 시간 기록, 안정성·과거 갱신 시험과 자료 보존
- 사용자 용량 관리 요청: `daily`의 검토·Git 근거 확인 경로만 정리
- 중복 제외 20GiB 상한·미검토·만료 점검, 시험 기본 `CARGO_INCREMENTAL=0`과 명시 설정 보존
- 실행 중·목록 변경·연결 경로 삭제 거절, 작은 결과 Markdown의 Git 보존
