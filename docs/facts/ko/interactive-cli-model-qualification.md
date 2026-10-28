---
schema_version: 1
pair_id: interactive-cli-model-qualification
topic_slug: interactive-cli-model-qualification
language: ko
counterpart: ../en/interactive-cli-model-qualification.md
title: 대화형 CLI 모델 수용
summary: 실제 모델 계측과 지속 설정 회귀 검사의 증명 범위 구분
tags:
- interactive-cli
- model-evaluation
- preservation
- regression
aliases: []
sources:
- repo:docs/research/cli-model-validation-0.14.0.md#sha256:13d1c983971e95ec5b8a7b927451a4272f6215c63ab86f9d8da3b2ed8652807e
- repo:tests/conformance/contracts/test_setup_summary.py#sha256:67090516ace0ed798c6f1388ddc7fc9ab35ce4815fdd801435f126aa7a96f401
- repo:tests/fixtures/interactive-cli/model-fixed-choices.json#sha256:89b5913b061ecb20908a1ea47c0fe7cd61206c3f49c6fb7e4077735f52cc8f38
- repo:tests/fixtures/interactive-cli/model-project-receipt.py#sha256:2a3c75ff8e8b01df54958cd5718def0ab6786e2b483ad057362eb7456370dee3
links:
- consumer-session-coordination
- project-onboarding
reviewed_revision: git:cd1a9dd8df64e42e6fcc8bdd3e24d4941987afa7
status: active
---

# 대화형 CLI 모델 수용

0.14.0의 Windows Codex 시험에서 판/범위별2회 계측. Hive 입력 바이트 중앙값 감소는 전역60.7038%·프로젝트50.0094%, 직접 CLI의 모델 호출0회. 토큰·요금·처리 속도 증명 제외, 프로젝트 기준 충족 여유3바이트.

사용자 요청은 고정 입력의 실제 CLI 회귀를 develop 통합 검사에 보존하고, 동일 매개변수별 최대5회 적용. 실패/진행 중 예약 포함, 임시 경로·문서 커밋만의 변경으로 횟수 초기화 금지. 실제 모델/입력 변경은 별도 조건, 전체 누적 횟수는 승인 기준 제외.
