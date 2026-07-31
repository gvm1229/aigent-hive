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
  - "repo:docs/releases/0.8.0.md#sha256:1d5100af5c1f8b2d9e19d2a730acdcff9d1fa276c0cfdb364ec9b33164b78205"
links: [global-onboarding, version-policy, windows-powershell-module-isolation]
reviewed_revision: "git:e37de7ff99fb235f673a4d3273deb54d6284999e"
status: active
---

# npm 0.8.0 배포

후보 실행 `30657669889`에서 보호된 `develop` 커밋 `420e244` 검증 완료. 게시 실행
`30658188721`은 npm 패키지 여섯 개를 exact `0.8.0|latest`로 게시하고, 변경할 수
없는 기존 `0.8.0-test.1|test` 보존. npm과 Windows 직접 설치 바이너리의
SHA-256은
`330f4e0c8da5b6347400b9b16a9f76b2fb4f94406a2eacfe8c641367ca344ef9`로 동일.
GitHub Release와 Git 출시 태그 생성 없음. 요청 배경: npm 설치에 유효한
`latest` 경로가 필요하므로 관리자가 exact `0.8.0` 게시 승인.
