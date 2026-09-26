---
schema_version: 1
pair_id: v0-11-stable-release
topic_slug: v0-11-stable-release
language: ko
counterpart: ../en/v0-11-stable-release.md
title: "0.11.0 안정판 출시"
summary: "공개 시험판 test.9 수용 뒤 정확한 main 커밋의 동일 제품으로 0.11.0 안정판 출시"
tags: [release, stable, v0-11]
aliases: ["0.11.0 stable"]
sources:
  - "repo:docs/research/0.11.0-stable-release-2026-09-27.md#sha256:bd5ee6d27564a184e7ad164b22510bb55477ed0aa100d535fdb3e4dd313dab68"
links: [release-verification, stable-public-documentation]
reviewed_revision: "git:ed6d8f0aee79b73bb83b4e9a057be85af304c1f0"
status: active
---

# 0.11.0 안정판 출시

- 공개 test.9 제품 바이트를 Windows·macOS·Linux에서 수용, PR #64로 develop에서 main 통합
- 후보 36269801223: main 커밋 ed6d8f0a의 네이티브 대상 5개와 npm 패키지 6개 생성
- 첫 게시의 npm 태그 전파 확인 실패는 실패 기록으로 보존. 복구 36271127837이 동일 바이트 검증 뒤 GitHub 정식 출시·Discord 단계 완료
- npm 6개 패키지의 정확한 0.11.0·latest 확인, 공식 태그 v0.11.0은 ed6d8f0a로 연결
- 안정판을 회귀 시험용으로 설치한 기록 제외
