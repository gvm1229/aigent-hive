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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:05cc3c9e69e277d272d3aa7e372ab20a5da2cd8f5f1b21c45f03d9d6eda9607c"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:4150391c45d8276bafb14db9117aae13e9f87dc15708b10d5c609654e0b0767a"
  - "repo:crates/hive-render/src/lib.rs#sha256:d037c1cea2d7a4bef6b5476a0cac77a8b7113034ec135cc5043fc7e30416cf95"
  - "repo:docs/archive/plans/releases/0.9.5/release-0.9.5-stable-publication.md#sha256:70ed823701fa0ae8be728d97b8705846f0eaa50e6e8758425d439bfee4d1334c"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
  - "repo:scripts/check-project-base-coverage.py#sha256:9fb5bf18a2bc89f0f990c89d5dda633455e22fa5e3fbf527f83f08ae0519dff2"
  - "repo:scripts/qualify-project-predecessors.py#sha256:4b40b7a8fd7800b0294db2d4addedc825443a9e10db87eea0d376d29ed591918"
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
