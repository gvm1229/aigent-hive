---
schema_version: 1
pair_id: usage-stop-notice
topic_slug: usage-stop-notice
language: ko
counterpart: ../en/usage-stop-notice.md
title: "사용량 보호 중단 안내"
summary: "제품 안내 창의 원인·상태·재개 설명과 실제 Windows 표시 검증"
tags: [notice, usage, windows]
aliases: []
sources:
  - "repo:docs/research/usage-stop-notice-0.12.1.md#sha256:e4bb6407c618423280f980006c64aa6c6c2eb17f52d2fee44df5abe8552af36b"
  - "repo:tests/results/usage-stop-notice-windows-0.12.1.json#sha256:406e5d0573cd42c27dcf980b5517e5f29ff324baaefa849c47ee6c6407b4b2c5"
links: [installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:ee69dbe35105053232d9029e50007519f6052fbe"
status: active
---

# 사용량 보호 중단 안내

- 0.12.1 제품의 별도 Windows 안내 창. Hive의 중단 원인·확인 상태·앱 유지·직접 재개 방법 표시
- 실제 제품의 시험 표식 제외. 닫기는 창만 닫는 동작, 사건 확인·보호 해제·작업 재개 없음
- 초기화·60초 조회 공백·부분 확인·읽기 실패·확인 완료의 구분, 원래 이유 보존
- Windows 144 DPI·560/460 폭 표시, 진입875ms·접근성 닫기·Esc 경로·사건 보존 통과
- 표시만 검증, 실제 중단·설치와 다른 OS·96/192 DPI·실제 키보드 입력의 증명 제외
