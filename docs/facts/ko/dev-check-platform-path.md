---
schema_version: 1
pair_id: dev-check-platform-path
topic_slug: dev-check-platform-path
language: ko
counterpart: ../en/dev-check-platform-path.md
title: "dev-check 플랫폼 PATH"
summary: "호스트 전용 pathlib 클래스 생성 없는 사전 푸시 도구 PATH 구성."
tags: [development, portability, verification]
aliases: ["dev-check PATH 이식성"]
sources:
  - "repo:scripts/dev-check.py#sha256:6ba74f6a9b9f262b11046e3f2003285839b447e55b2f03aeafc62f054e1690ba"
links: [release-verification]
reviewed_revision: "git:39569b7a2a7c67f8ab19010db8c4df32da470f86"
status: active
---

# dev-check 플랫폼 PATH

사전 푸시 실행기의 도구 디렉터리 계산: 문자열 기반 운영체제 경로 연산.
효과: `WindowsPath` 생성 불가 비-Windows 호스트에서도 Windows 모드 검증 가능.
수용 기준: 모의 Windows 기본 모드를 포함한 `test_dev_check` 시험 전체 완료.
기록 배경: 사용자 요청 Hive-native orchestration 계획의 게시 적격성 검증.
