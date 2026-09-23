---
schema_version: 1
pair_id: historical-project-base-coverage
topic_slug: historical-project-base-coverage
language: ko
counterpart: ../en/historical-project-base-coverage.md
title: "과거 프로젝트 기준본 수용 범위"
summary: "선언된 프로젝트 갱신 source range와 exact full 기준본·matrix 수용의 대응"
tags: [migration, project-upgrade, regression, release]
aliases: ["과거 기준본 정합성"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:08d4aa0959ccc377a3f96a4c6f37df6f71c1473a271b406f7eb3b214f860cf0c"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:8bc640b86bebf10156d116482e1fd10f5504af96ffe1cfab093cdbd6b1594812"
  - "repo:crates/hive-render/src/lib.rs#sha256:87415202d29e198529a2d39fa256b33ded6ec41c0e34a45bb6d252e0393e74c2"
  - "repo:docs/archive/plans/releases/0.9.5/release-0.9.5-stable-publication.md#sha256:70ed823701fa0ae8be728d97b8705846f0eaa50e6e8758425d439bfee4d1334c"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
  - "repo:scripts/check-project-base-coverage.py#sha256:6c90b2a4b1f84507f56a80ed540f6e97c9938b6f91a7ab087cc8347c8cfadf26"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:0340f8a26d14ebcc134c14e421d605f8022912f8"
status: active
---

# 과거 프로젝트 기준본 수용 범위

- `0.9.2` 과거 marker: 저장된 Markdown backend 재현
- local override: 한 번의 적용 뒤 current 기준 수렴
- PortareFolium 읽기 전용 copy: 검사·미리 보기·적용·검증, local marker·외부 파일 보존, 변조 ledger 무변경 실패 통과
- 과거 `REL95-004`: 공개 `0.9.5-test.15` 갱신 수용 완료, `test.4` 대기 상태 대체
- 과거 성공과 `0.11.0` 신규 수용의 별도 판정
