---
schema_version: 1
pair_id: usage-stop-notice
topic_slug: usage-stop-notice
language: ko
counterpart: ../en/usage-stop-notice.md
title: "사용량 보호 중단 안내"
summary: "공식 로고와 닫기만 제공하는 Windows 알림"
tags: [notice, usage, windows]
aliases: []
sources:
  - "repo:docs/research/usage-stop-notice-0.12.1.md#sha256:58148052d858dec443c5d8231c0ad91c7748dfd8e54e2a5b5939e6fc5790cffd"
  - "repo:tests/results/usage-stop-notice-only-windows-0.12.1.json#sha256:89acbc9c15cf5c0988ff69f823cdee69894f07957898d4f80d689609b64e8c4c"
  - "repo:tests/results/usage-stop-notice-windows-0.12.1.json#sha256:12746751e84b05243501dab68a86f2ba228424ac13d00fb12f9768c0ceefadf0"
links: [installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:da2797509ea7e0ef94783877e3c2cb69c6bfd490"
status: active
---

# 사용량 보호 중단 안내

- 0.12.1 Windows 알림 전용 창: 공식 Hive 로고·중단 이유·확인 상태·앱 유지·감지 시각·Codex 직접 재개 안내
- 닫기만 제공. 명령 복사·사건 확인·잠금·해제·작업 실행 기능 없음, 창 닫기의 사건 보존·자동 재개 없음
- 초기화·60초 조회 공백·부분 확인·읽기 실패의 구분. 현재 Windows 144 DPI·560/460 폭·13개 회귀·실제 표시·접근성 닫기와 원래 Codex 앱 유지 확인
- 추가 실제 중단·실제 키보드·96/192 DPI·다른 OS의 증명 제외
