---
schema_version: 1
pair_id: directive-context-recovery
topic_slug: directive-context-recovery
language: ko
counterpart: ../en/directive-context-recovery.md
title: "반복 압축 뒤 지침 복구"
summary: "승인 구절 복원 구현과 실제 압축 준수 검증의 분리"
tags: [context, hooks, policy]
aliases: []
sources:
  - "repo:docs/guides/directive-context-recovery.md#sha256:53c7c389d0fb7d20c61597bdb1388aa4c89c30f2871754eb51f835e9f424a8ee"
links: [project-policy-enforcement]
reviewed_revision: "git:7539f44b947b1fe0c90ea3d5c083022c99c9e341"
status: active
---

# 반복 압축 뒤 지침 복구

- 0.11.0 구현: 승인한 Markdown 구절 복원·경로별 안내·명시적 파일 보호, 출력 4,096바이트 상한
- 매 압축 복원, 부모 세션 전달 상태 공유와 추가 모델 호출 제외
- 바뀐 원문 자동 주입 금지. 승인 설정 손상 시 파일 검사 거부, 철회 경로 유지
- Windows CLI 재생은 전달·차단 검증. 실제 호스트 압축·모델 준수는 별도 미검증
- 사용자 목표: 반복 압축 뒤 지침 유지와 불필요한 강제·토큰 사용 최소화
