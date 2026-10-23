---
schema_version: 1
pair_id: usage-default-off-scope
topic_slug: usage-default-off-scope
language: ko
counterpart: ../en/usage-default-off-scope.md
title: "사용량 보호·대화형 CLI의 확정 범위"
summary: "전역·프로젝트 대화형 설정과 명시적 사용량 선택을 포함한0.14.0 계획"
tags: [guard, planning, usage]
aliases: []
sources:
  - "repo:docs/decisions/interactive-cli-0.14.0.md#sha256:bd680e226a4a3391750ae3d2cd921fbaa3c4b36f298d6b61ad9f88dfd452aef7"
  - "repo:docs/plans/0.14.0-implementation.md#sha256:8f818ae7be8db3539f5722d6aa257a068999604f701340229101d7b476093f32"
  - "repo:docs/research/interactive-cli-0.14.0.md#sha256:ffd6dd3695c3ef9128e006d2191d93460fa177856b2c90c6621ce3c7ddc4297a"
links: [usage-guard-stop-layers]
reviewed_revision: "git:c8aeb4cb1dd2c3f5e18c2f94c6c4f9c20d02732c"
status: active
---

# 사용량 보호·대화형 CLI의 확정 범위

0.14.0 계획: 전역·프로젝트 대화형 설정, 공개0.13.1 첫 갱신의 질문, 로컬 설정 출처 기록. 신규 사용량 중단 기본 비활성. 기존 활성·출처 불명은 CLI에서 확인, 비대화형 갱신은 선택 보존·질문 보류. 제거 미리보기·확인·자료 보존, 전체 삭제 제외. 직접 CLI의 모델 호출0회와 AI 입력 감축·소형 모델 검증이 수용 기준. 제품 구현 전의 저장된 계획, 다음 공개 시험0.14.0-test.1.
