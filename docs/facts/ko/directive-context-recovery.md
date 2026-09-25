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
  - "repo:docs/guides/directive-context-recovery.md#sha256:20a1f22a3570c4dd8ad993c0ae04fee31ebcbc43646f90c72811501d30ea0438"
  - "repo:docs/research/directive-context-implementation-0.11.0.md#sha256:eb38786a9b205550a61a3a9a52f98aac7766b04999dbd31fcdd8bd51e6da2e6e"
links: [project-policy-enforcement]
reviewed_revision: "git:1d36069531a894895a3517a2937666069cfe346e"
status: active
---

# 반복 압축 뒤 지침 복구

- 0.11.0-test.8: 승인 Markdown 구절 복원·경로별 안내·보호 파일 검사, 4,096바이트 상한
- 매 압축·하위 작업 시작에 재전달. 부모 세션 전달 상태 공유와 추가 모델 호출 제외
- 바뀐 원문 자동 주입 금지. 설정 손상 시 파일 검사 거부·철회 가능
- 공개 실행 파일의 Windows·macOS·Linux 훅 계약 각각 26개 통과
- 실제 Codex 압축·모델 준수·사용량 비교는 활성화 승인 대기
- 목표: 반복 압축 뒤 지침 유지, 불필요한 강제·토큰 사용 최소화
