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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:d5e36f5d1cb6080fa7952b1cf4354e7d54f0df12bc0799d758ced53d7f083b84"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:d3bbe15942fc1ebdd38a81f62b37e9da2e47ac5db74259ca846fd1e1f08a3ac8"
  - "repo:crates/hive-render/src/lib.rs#sha256:9b9fd4e3a9734087d452f85e02a21904f60d6233de8bbc566459d4949bf13202"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
  - "repo:scripts/check-project-base-coverage.py#sha256:78f471770ca909180c73ac1e65783f0a9ebd762a12ad70411575197578459ce3"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:47d4663f1e1f263276f9ce54b7c69a3ff95d2170"
status: active
---

# 과거 프로젝트 기준본 수용 범위

- `0.9.2` 과거 marker: 저장된 Markdown backend 재현
- local override: 한 번의 적용 뒤 current 기준 수렴
- PortareFolium 읽기 전용 copy: 검사·미리 보기·적용·검증, local marker·외부 파일 보존, 변조 ledger 무변경 실패 통과
- 공개 artifact 수용: `0.9.5-test.4` 대기
