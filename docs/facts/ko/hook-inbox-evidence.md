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
  - "repo:docs/research/overhaul/OV150-001.md#sha256:e4aca1efb604fa5ca8f2a993142f86c7be14add5395c9db1f2a2634ba9fbd46a"
links: [orchestration-ownership]
reviewed_revision: "git:7a7bf6321c47e671c9b28f77c661b60f0302eeb2"
status: active
---

# 훅 수신 상자 검증 경계

OV150-001: Windows 호스트 처리기의 파일 봉투 소비·구독 로그인 Codex 실행 경로 확인. 합성 훅 입력의 증명 범위는 처리기 구간만 해당. 전체 완료에는 공식 훅 반환을 받은 대화형 Claude의 일치하는 결과·확인증 보고 필요. 비대화형 Claude·저장된 반환 JSON·작업자 종료 코드만으로 대체 금지.
