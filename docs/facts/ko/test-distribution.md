---
schema_version: 1
pair_id: test-distribution
topic_slug: test-distribution
language: ko
counterpart: ../en/test-distribution.md
title: "npm 0.8.0 배포"
summary: "npm exact 0.8.0의 latest 설치 경로 제공, GitHub Release·Git release tag 생성 없음"
tags: [distribution, release, test]
aliases: ["0.8.0 release scope"]
sources:
  - "repo:Cargo.toml#sha256:5083784d829c1e5ee6e642b54a3e616e78327dc1b8deb139bc00f8d14374b830"
  - "repo:scripts/package-npm.mjs#sha256:7d286e69158752940c877ce7b8604ee336b7beb7ede0632871d9bae2e9546710"
links: [global-onboarding, version-policy, windows-powershell-module-isolation]
reviewed_revision: "git:cdde668bed5f3b35e08a35f64e7e25594ce9c3a2"
status: active
---

# npm 0.8.0 배포

npm exact `0.8.0`을 `latest`로 게시. `npm install -g aigent-hive`와 exact
`@0.8.0`은 같은 패키지 계열 사용. 기존 `0.8.0-test.1|test`는 이전 검증 이력으로
보존. GitHub Release·Git release tag 생성 없음. 산출물은 보호된 `develop`의 정확한
후보에서 만들고 npm 검증 성공 뒤 같은 커밋을 pull request로 `main`에 반영.
완료 기준: 제품·패키지 버전 일치, 플랫폼 의존 버전 일치, provenance, byte identity,
포장·작업 흐름 시험 통과. 요청 배경: npm의 유효한 `latest` 경로가 필요하므로
태그가 붙지 않은 npm `0.8.0` 배포 승인.
