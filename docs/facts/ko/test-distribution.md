---
schema_version: 1
pair_id: test-distribution
topic_slug: test-distribution
language: ko
counterpart: ../en/test-distribution.md
title: "0.8.0 시험 배포"
summary: "제품 후보 0.8.0과 npm 시험판 0.8.0-test.N을 분리하며 GitHub Release·npm latest 이동 없음."
tags: [distribution, release, test]
aliases: ["0.8.0 release scope"]
sources:
  - "repo:Cargo.toml#sha256:5083784d829c1e5ee6e642b54a3e616e78327dc1b8deb139bc00f8d14374b830"
  - "repo:scripts/package-npm.mjs#sha256:22c8a4e6b71764d2c3987a3525736d7406ce2a0d6da75ed96da420996a4d2e2c"
links: [global-onboarding, version-policy]
reviewed_revision: "git:b74afdae66f2704c6b24e42d47332ed931e2fecd"
status: active
---

# 0.8.0 시험 배포

제품 후보 `0.8.0`의 배포일은 `2026-07-31`, 첫 npm 운반판은
`0.8.0-test.1|test`. GitHub Release·안정 npm `0.8.0`·`latest` 이동 없음.
산출물은 보호된 `develop`의 정확한 커밋에서 생성하고, npm·직접 설치 검증 뒤
같은 커밋을 pull request로 `main`에 반영. 완료 기준은 제품·포장판 입력 분리,
정확한 명세, 플랫폼 의존 버전 일치, 포장·작업 흐름 시험 통과. 요청 배경은
공개 승인 전 반복 가능한 설치 시험 제공.
