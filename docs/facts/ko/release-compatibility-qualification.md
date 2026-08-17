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
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:c5dba7810327a88235025ea62ba2b77387a072c8e76b044b661ddb911aa26220"
  - "repo:docs/plans/active/release-0.9.5-stable-publication.md#sha256:eb745caa379f293cfb71acb07a680a840765d1c3d803e70473110a8e3f3f9c22"
  - "repo:docs/plans/active/release-compatibility-qualification-0.9.5.md#sha256:fde4ef5a0a738761093a40f84f93d0a6eb4e6a5e64449f606b71ef6619cb20da"
  - "repo:scripts/check-project-base-coverage.py#sha256:b7d0887ccd3e3a9019383c3cea283189361c17fa62d806f44096cd6825b67579"
links: [historical-project-base-coverage, release-verification, test-lane-inventory]
reviewed_revision: "git:0fd5ea87fa377dc584dcfa6ad93ae9ee74eb4e97"
status: active
---

# 출시 호환성 수용

- `0.9.5-test.3` candidate·공개 시험판: source `224170e`, exact npm 시험 package, Windows archive 다이제스트 결속
- Windows 격리 경로: public direct installer와 npm `0.9.4` 업그레이드 성공
- user projection·bare update·public `0.9.2` project upgrade: 인증 불가 기존 ownership으로 중단, stable promotion 우회 금지
