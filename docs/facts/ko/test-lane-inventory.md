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
  - "repo:docs/guides/test-lanes.md#sha256:f411a47fa291833172ecf56219e0446b806b84c49206ceed926279bf27d17141"
  - "repo:scripts/test-lanes.py#sha256:5bc7694c5e1f399880069d16edbde37b85c741dadc5d6252892ebd5142cea8b1"
  - "repo:scripts/test_artifacts.py#sha256:d5aa3c82a7d7aaf76eee072ac675ed8d04990aed61451e329bfdf798f9e88785"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:df8aa9994a9fa02a4ee782567f646f664d7414ca244aa679e49498a7832b041f"
  - "repo:tests/conformance/integration/test_connected_setup_lifecycle.py#sha256:f74ae9ecf4d442e4171b4f0b28bb4d2a7ad75167858d8cba436e9710021e12ab"
  - "repo:tests/conformance/lanes.toml#sha256:28e9d1ab7c0edb9325c4f923708982f21be0f963395f8cc27ca67df38abd065a"
links: [release-verification, test-fault-isolation]
reviewed_revision: "git:0340f8a26d14ebcc134c14e421d605f8022912f8"
status: active
---

# 시험 lane 대장

Python 시험·fixture: phase 디렉터리 대신 목적별 package 사용.
`tests/conformance/lanes.toml`: 재귀 발견한 모든 `test_*.py` 모듈의 documentation·security·
contract·integration·release 중 하나 배정. 실행기: 누락·중복 거부, 변경 경로 기반 lane 선택,
module 시간 JSON 제공. 생성 산출물 정리 전 실행 결과 Markdown 보존. 재편 중 안정성·과거
upgrade 시험과 fixture 삭제 `0건`.
