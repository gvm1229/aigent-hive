---
schema_version: 1
pair_id: usage-default-off-scope
topic_slug: usage-default-off-scope
language: ko
counterpart: ../en/usage-default-off-scope.md
title: "사용량 보호·대화형 CLI 범위"
summary: "0.14.0 CLI 구현의 제한된 근거와 전체 운영체제·모델·공개 파일 수용 구분"
tags: [guard, planning, usage]
aliases: []
sources:
  - "repo:crates/hive-core/src/preferences.rs#sha256:838005e60b3255422c2ee143d76e66ef3ef877d7f3a8ff196eb446c0dfdc888d"
  - "repo:docs/decisions/interactive-cli-0.14.0.md#sha256:40e381b151d7da2535c5c2c0a956202e26edb23567a081f4782b6c420939baf7"
  - "repo:docs/plans/0.14.0-implementation.md#sha256:685d7db95252c69c24f49f393b2e53350cc3506bd5543e21346bb3a26c8bed5b"
  - "repo:docs/research/cli-usage-settings-0.14.0.md#sha256:135a213351bf10e7d4ac861da0a2ead5af4e881fd375cf81a8a9fe1c1401d07e"
  - "repo:docs/research/cli-user-settings-0.14.0.md#sha256:029f15d8f36dbc94ad6e63444e3ba8ad22ae8ef1566a0f2a6080d77bb110f636"
  - "repo:docs/research/interactive-cli-0.14.0.md#sha256:ffd6dd3695c3ef9128e006d2191d93460fa177856b2c90c6621ce3c7ddc4297a"
links: [usage-guard-stop-layers]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# 사용량 보호·대화형 CLI 범위

0.14.0의 전역·프로젝트 로컬 설정, 공개0.13.1 첫 갱신 질문과 설정 출처 기록. 신규 자동 중단 비활성, 기존 활성·출처 불명의 CLI 재검토. 비대화형 갱신의 선택 보존·질문 보류. 제거 미리보기·확인·자료 보존, 전체 삭제 제외. 직접 CLI 모델 호출0회, AI 입력 감축·Luna 평가는 실제 계측 필요. 소스 구현과 제한된 CLI 근거 확보, 전체 OS·모델·공개 파일 수용 미완료. 다음 공개 시험0.14.0-test.1.
