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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:0827c9c337e692cd64767fca05256862950fbb1b765595a3d7431ff75d265380"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:f5c90bf5b90baef8d7a5ec2228d0d23338ea95aef2c8078b7656c4df7d6ff600"
  - "repo:crates/hive-render/src/lib.rs#sha256:9a8dd35a7cbd20a71c44e5a09330410bf45bdd68de04c623454187d706687449"
  - "repo:docs/archive/plans/releases/0.9.5/release-0.9.5-stable-publication.md#sha256:70ed823701fa0ae8be728d97b8705846f0eaa50e6e8758425d439bfee4d1334c"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
  - "repo:scripts/check-project-base-coverage.py#sha256:9fb5bf18a2bc89f0f990c89d5dda633455e22fa5e3fbf527f83f08ae0519dff2"
  - "repo:scripts/qualify-project-predecessors.py#sha256:80f839a04103dc25d74259acd5b1daa7d933fa58e98eb605206eb1a046c67f58"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:cace7e3fa885dd20d1b7a068b45c7f1536503d60"
status: active
---

# 과거 프로젝트 기준본 수용 범위

0.9.2 표시 블록은 저장된 Markdown 백엔드 사용, 한 번 적용으로 사용자 변경 기록.
읽기 전용 PortareFolium 사본에서 조사·미리보기·적용·검사·원본 보존·변조 거부 통과.
REL95-004에 0.9.5-test.15 공개 수용 보존, test.4 대기 해소. 과거 근거로 0.11.0 수용 대체 금지.

0.9.1 이후 모든 공개 안정판에 갱신 경로와 실제 구버전 CLI 표본 필수. 누락 시 출시 후보 검사 실패.
Windows에서 0.10.3까지 아홉 버전의 0.11.0 갱신·사용자 내용 보존·실패 원복·중단 복구 통과.
모든 과거 설정 조합 증명은 제외. 공개 실행 파일의 실패 주입 기능 제외.
