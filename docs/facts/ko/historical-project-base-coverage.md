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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:af09aadf2ddfabc082dfac9ae6c8233c2fe48f964db8996063848838f04f68c5"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:5d1ded97d4dfa1fcc3bbac149ededed530ce4d384eb8b87b360c441fbbce8deb"
  - "repo:crates/hive-render/src/lib.rs#sha256:644c0b46c68ceaeb9cb798f2c076f301ed12be889121cd4c086f23ecd50e69ae"
  - "repo:scripts/accept-public-hive.py#sha256:59a78bea773c38e18248fb6cdefe6e612a69d8f46ae0139eeff7a7b30fa455f2"
  - "repo:scripts/check-project-base-coverage.py#sha256:b7d0887ccd3e3a9019383c3cea283189361c17fa62d806f44096cd6825b67579"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:32bf5dfd2cd2663070174a4efebee39d7fa98935"
status: active
---

# 과거 프로젝트 기준본 수용 범위

- `0.9.2` 과거 marker: 저장된 Markdown backend 재현
- local override: 한 번의 적용 뒤 current 기준 수렴
- PortareFolium 읽기 전용 copy: 검사·미리 보기·적용·검증, local marker·외부 파일 보존, 변조 ledger 무변경 실패 통과
- 공개 artifact 수용: `0.9.5-test.4` 대기
