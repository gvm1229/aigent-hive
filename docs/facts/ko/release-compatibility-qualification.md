---
schema_version: 1
pair_id: release-compatibility-qualification
topic_slug: release-compatibility-qualification
language: ko
counterpart: ../en/release-compatibility-qualification.md
title: "출시 호환성 수용"
summary: "0.9.5 계획의 declared compatibility source 전수 executable matrix와 stable promotion 전 evidence gate"
tags: [compatibility, migration, release, testing]
aliases: ["호환성 matrix gate"]
sources:
  - "repo:.github/workflows/release.yml#sha256:53f1b3c4284326ae392594d1135ad600ddbd8035ec2caf16ed9ea0e52dc2efd4"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:5d1ded97d4dfa1fcc3bbac149ededed530ce4d384eb8b87b360c441fbbce8deb"
  - "repo:docs/plans/active/release-0.9.5-stable-publication.md#sha256:7a4fa8bee33908d623acc312c9e8c9431d578bca08306e0711b514347a1e733b"
  - "repo:docs/plans/active/release-compatibility-qualification-0.9.5.md#sha256:722d961e65b3ed28b344ce2fc27edb1f08453f738cb5c33b11e920ae15c53429"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
  - "repo:scripts/check-project-base-coverage.py#sha256:b7d0887ccd3e3a9019383c3cea283189361c17fa62d806f44096cd6825b67579"
links: [historical-project-base-coverage, release-verification, test-lane-inventory]
reviewed_revision: "git:eaf0126b6abd73427dc01cd02d21eeda9a7736e5"
status: active
---

# 출시 호환성 수용

- 수정 updater의 공개 직접 설치 수용: Windows `test.12 → test.13`, macOS `test.14 → test.15` 성공
- public `test.15`의 PortareFolium `0.9.2` upgrade matrix: source·foreign byte 보존과 tampered ledger
  no-mutation 성공
- protected `main` integration과 다섯 플랫폼 stable candidate 성공
- 남은 항목: 안정판 npm publication과 post-publication validation. 별도 go sign 전 금지
