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
  - "repo:scripts/qualify-project-predecessors.py#sha256:c4e75d248a201b433c01423436645e32920543d58360dbd2b098f9fc3d909278"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:4c70291742b0856063acb218aebe27b85138cca5"
status: active
---

# 과거 프로젝트 기준본 수용 범위

0.9.2 표시 블록은 저장된 Markdown 백엔드 사용, 한 번 적용으로 사용자 변경 기록.
읽기 전용 PortareFolium 사본에서 조사·미리보기·적용·검사·원본 보존·변조 거부 통과.
REL95-004에 0.9.5-test.15 공개 수용 보존, test.4 대기 해소. 과거 근거로 0.11.0 수용 대체 금지.

0.9.1 이후 모든 공개 안정판에 갱신 경로와 실제 구버전 CLI 표본 필수. 누락 시 출시 후보 검사 실패.
Windows에서 0.10.3까지 아홉 버전의 0.11.0 갱신·사용자 내용 보존·실패 원복·중단 복구 통과.
모든 과거 설정 조합 증명은 제외. 공개 실행 파일의 실패 주입 기능 제외.
