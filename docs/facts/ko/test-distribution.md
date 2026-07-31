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
  - "repo:scripts/package-npm.mjs#sha256:22c8a4e6b71764d2c3987a3525736d7406ce2a0d6da75ed96da420996a4d2e2c"
links: [global-onboarding, version-policy]
reviewed_revision: "git:3143c0e90b3c474c739651f7ddc2350bbf5e020a"
status: active
---

# 0.8.0 시험 배포

제품 후보 버전은 `0.8.0`, npm 운반 버전은 `0.8.0-test.N`으로 분리하며
첫 시험판은 `0.8.0-test.1`. `test` 태그로 `aigent-hive@test` 설치 가능.
안정 npm `0.8.0` 사용, GitHub Release 생성, npm `latest` 이동 없음.
후보 산출물은 보호된 `develop`의 정확한 커밋에서만 생성. npm·직접 설치기
검증 성공 후 같은 커밋을 pull request로 `main`에 반영. 완료 기준은 제품·패키지
입력 분리, 정확한 manifest 메타데이터, 플랫폼 의존 버전 일치, 포장·배포 작업
시험 통과. 요청 배경은 실제 공개 승인 전 반복 가능한 설치 시험 제공.
