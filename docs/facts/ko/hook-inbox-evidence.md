---
schema_version: 1
pair_id: hook-inbox-evidence
topic_slug: hook-inbox-evidence
language: ko
counterpart: ../en/hook-inbox-evidence.md
title: "훅 수신 상자 검증 경계"
summary: "작업자 실행과 대화형 Claude 결과 수신의 별도 근거 필요"
tags: [evidence, hooks, orchestration]
aliases: [hook-inbox-evidence]
sources:
  - "repo:docs/research/overhaul/OV150-001.md#sha256:feaf7d18aa48c26b31ac5210cac88b3a916ea80ff29106de97b3ab55d65329e6"
links: [orchestration-ownership]
reviewed_revision: "git:eb988a3637ec5ce3035de4cd758b133828d3c5c0"
status: active
---

# 훅 수신 상자 검증 경계

OV150-001: Windows 호스트 처리기의 파일 봉투 소비·구독 로그인 Codex 실행 경로 확인. 합성 훅 입력의 증명 범위는 처리기 구간만 해당. 전체 완료에는 공식 훅 반환을 받은 대화형 Claude의 일치하는 결과·확인증 보고 필요. 비대화형 Claude·저장된 반환 JSON·작업자 종료 코드만으로 대체 금지.

어시스턴트 응답 없는 UserPromptSubmit 발생 가능. 대기 파일만으로 Stop·작업자 실행 증명 불가. 출력 없는 Stop의 실행 소비권 획득 금지·대기 요청 복구 가능 상태 유지.
