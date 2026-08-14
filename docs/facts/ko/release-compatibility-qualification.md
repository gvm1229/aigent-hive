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
  - "repo:.github/workflows/release.yml#sha256:3f92bd0fb18bf5519493389af3edcaf44a6d81e0c1c9fdc0382cf4a6931c7f6c"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:0aa8c272002f64443b8204e80f5744c02474e4621ca807d28cfe36ff3bdb49f6"
  - "repo:docs/plans/active/release-compatibility-qualification-0.9.5.md#sha256:edb53b6054f7d0f17a09e7331ff15e15aa02d630a1adbf483b954b400d83f247"
  - "repo:scripts/check-project-base-coverage.py#sha256:b7d0887ccd3e3a9019383c3cea283189361c17fa62d806f44096cd6825b67579"
links: [historical-project-base-coverage, release-verification, test-lane-inventory]
reviewed_revision: "git:e44b9498ca36ebeb0b477a3d4e5c06a4e71561ef"
status: active
---

# 출시 호환성 수용

`0.9.5` 로컬 수용에는 다이제스트 결합 프로젝트 기준본 수용 보고서와 이전 프로젝트 상태용 컴파일 명령줄
도구 수용 행렬을 더한다. 향후 후보 작업 흐름은 이 보고서를 산출물로 보관한다. 공개 시험과 안정판 게시는
관리자 결정에 따라 보류한다. 재개할 때 산출물과 수용 보고서 다이제스트가 일치하지 않으면 승격을 중단한다.
