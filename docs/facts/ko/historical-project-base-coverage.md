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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:81c8223df0cba01511f0da69e8bc2b79bca26a22fa0439af874f6b4ed320606e"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:947b34850968810294cdfacf7b3c9b84540916597be7ea3a419589cd62830a8f"
  - "repo:crates/hive-render/src/lib.rs#sha256:0beac46a96e59324d8531733df41dc13ab282b9e29cbf3c4cbea3454ae83df9c"
  - "repo:docs/archive/plans/releases/0.9.5/release-0.9.5-stable-publication.md#sha256:70ed823701fa0ae8be728d97b8705846f0eaa50e6e8758425d439bfee4d1334c"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
  - "repo:scripts/check-project-base-coverage.py#sha256:9fb5bf18a2bc89f0f990c89d5dda633455e22fa5e3fbf527f83f08ae0519dff2"
  - "repo:scripts/qualify-project-predecessors.py#sha256:670e7d1963f111e31e107a4a6a37225c5f2b9c28f406d9aafb4e718bed03cf02"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# 과거 프로젝트 기준본 수용 범위

0.9.2 표시 블록은 저장된 Markdown 백엔드 사용, 한 번 적용으로 사용자 변경 기록.
읽기 전용 PortareFolium 사본에서 조사·미리보기·적용·검사·원본 보존·변조 거부 통과.
REL95-004에 0.9.5-test.15 공개 수용 보존, test.4 대기 해소. 과거 근거로 0.11.0 수용 대체 금지.

0.9.1 이후 모든 공개 안정판에 갱신 경로와 실제 구버전 CLI 표본 필수. 누락 시 출시 후보 검사 실패.
Windows에서 0.10.3까지 아홉 버전의 0.11.0 갱신·사용자 내용 보존·실패 원복·중단 복구 통과.
모든 과거 설정 조합 증명은 제외. 공개 실행 파일의 실패 주입 기능 제외.
