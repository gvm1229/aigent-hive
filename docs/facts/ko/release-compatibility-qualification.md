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
  - "repo:docs/plans/active/release-0.9.5-stable-publication.md#sha256:859346ff9d42170f55d32c220c04baf84c9248f69077356bcbcf44abe13e5c38"
  - "repo:docs/plans/active/release-compatibility-qualification-0.9.5.md#sha256:722d961e65b3ed28b344ce2fc27edb1f08453f738cb5c33b11e920ae15c53429"
  - "repo:scripts/accept-public-hive.py#sha256:59a78bea773c38e18248fb6cdefe6e612a69d8f46ae0139eeff7a7b30fa455f2"
  - "repo:scripts/check-project-base-coverage.py#sha256:b7d0887ccd3e3a9019383c3cea283189361c17fa62d806f44096cd6825b67579"
links: [historical-project-base-coverage, release-verification, test-lane-inventory]
reviewed_revision: "git:b8e4c79437ea61cce0c012d37a8fed97860bf287"
status: active
---

# 출시 호환성 수용

- public `test.4`: actual `0.9.2` project copy upgrade 수용
- `test.5`: user projection 전 direct installer marker validation 결함 발견
- `test.6`: package·tag 공개 뒤 GitHub artifact·Release API HTTP 503
- 다음 수용: prerelease 복구, 이어서 `test.6 → test.7` user projection
- stable promotion: 두 공개 수용 전 금지
