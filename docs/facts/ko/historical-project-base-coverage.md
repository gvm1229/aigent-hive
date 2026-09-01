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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:17b4e24061b7214faa292fa50e65e9b0d9902270bdbe86fdc06ae53b7970bf05"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:f1b45ed3cfd4ae5feb40574c0825fbcc26efc67c95dc1032812656221a776f88"
  - "repo:crates/hive-render/src/lib.rs#sha256:58d45eb16a719523947a4ad6b50bc225a757aa2ca800ec95dbf957b74325803d"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
  - "repo:scripts/check-project-base-coverage.py#sha256:b7d0887ccd3e3a9019383c3cea283189361c17fa62d806f44096cd6825b67579"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:47d4663f1e1f263276f9ce54b7c69a3ff95d2170"
status: active
---

# 과거 프로젝트 기준본 수용 범위

- `0.9.2` 과거 marker: 저장된 Markdown backend 재현
- local override: 한 번의 적용 뒤 current 기준 수렴
- PortareFolium 읽기 전용 copy: 검사·미리 보기·적용·검증, local marker·외부 파일 보존, 변조 ledger 무변경 실패 통과
- 공개 artifact 수용: `0.9.5-test.4` 대기
