---
schema_version: 1
pair_id: codex-resume-lock-reference
topic_slug: codex-resume-lock-reference
language: ko
counterpart: ../en/codex-resume-lock-reference.md
title: "과거 Codex 재개 차단 재현"
summary: "현재 출시에서 제외한 반복 중단의 재현 자료"
tags: [codex, interruption, reference]
aliases: []
sources:
  - "repo:docs/research/codex-resume-lock-reference.md#sha256:1770e8379d94ab3ceae4c08c85e403ede1dba93c521e2ca7c88d3b96017bf3e9"
  - "repo:tests/results/session-adapter-latch-windows-0.12.1.json#sha256:f22b0a6a65fe3d2197972ef342a59fc67081c9dfb7c0dcd649c61db1672b0259"
links: [quota-reset-session-control, usage-stop-notice]
reviewed_revision: "git:6cee0fc8998d4b1b25eaed092f1a252b7144b210"
status: active
---

# 과거 Codex 재개 차단 재현

과거 잠금은 지속 사건·Hive 실행 거부·1초 간격 native 사용자 중단의 결합. 재개 뒤 시작한 새 실행에도 다시 중단 요청. 소스 전용 문서에 과거 커밋·단일 대화의 유한한 재현·새 관측과 정확 사건 확인을 전제로 한 해제·한계 보존. 단일 새 실행 136ms 확인·앱 유지 실측. 0.12.1-dev 검증본과0.13.0 공개 목표는 한 차례 중단·알림 전용 창, 재개 잠금 없음. 요청 범위는 미래 참고용 보존이며 활성화 제외.
