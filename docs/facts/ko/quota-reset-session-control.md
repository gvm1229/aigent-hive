---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: ko
counterpart: ../en/quota-reset-session-control.md
title: "현재 작업의 초기화 감지 전용 제어"
summary: "잠금 없는 한 차례 중단·직접 재개·Windows 검증과 로컬 감시"
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:836d4ac233d8e719a9cd1b5a09a1deee7111a7c4e208f95f07e277fa3c899b98"
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:09a27cf643e59088c6966d4a10e659069662860c8646db86b195f5bd1d13bafb"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:798f0e43b17f3446547899b0355cca2b177779889b1245ce4cec7dd0bfcc26de"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:da62eb6a722d291596514eb8a8eeb0ba5f58f80bfc503469c10318737db5e244"
  - "repo:docs/guides/installed-usage-guard.md#sha256:735fffd7960d1dca621b1de7bb2fa7f77ef232cda6c75c4165afda76c9f8da50"
  - "repo:docs/research/public-test-0.13.0.md#sha256:c831b239c282a19ee42a0cbf9779484f044bd973425d61f319e9c9947de7a2f9"
  - "repo:docs/research/release-0.13.0.md#sha256:fbdd1a5c8e306a8af4e54cc12306d9da35a41ba71ea7493adbc4de424be8f1a2"
  - "repo:docs/research/session-stop-0.12.1.md#sha256:50b5ba7baa1b9c2ed7bf96669115860655bb2fe0b7b5d7df7ac5fa628c650437"
  - "repo:docs/research/stable-release-0.13.0.md#sha256:75d8a1e6acf84fcb526d57d93166edfcfb52964591cf207fa733fe6ff3fa1b4b"
  - "repo:docs/research/usage-stop-notice-0.12.1.md#sha256:58148052d858dec443c5d8231c0ad91c7748dfd8e54e2a5b5939e6fc5790cffd"
  - "repo:tests/results/session-stop-one-shot-code-0.12.1.json#sha256:f97ceca1df94fff473e56240f8db49dce4ffe644b455154439008e67a3c21b56"
  - "repo:tests/results/session-stop-one-shot-installation-0.12.1.json#sha256:bbc233760b14fc02ebf35467618df2e59c26bab4e549806634a3c96d058d2371"
  - "repo:tests/results/session-stop-one-shot-qualification-0.12.1.json#sha256:000da296ad912dc0d30d30baed0f79f8a88193d6b09606bd75eff29e1dbe4406"
  - "repo:tests/results/session-stop-partial-windows-0.12.1.json#sha256:1f1d27dae9ef95f345c39fa018dce9f01b752cd765d24de3609b1ac7280a4071"
links: [automatic-dispatch-guard, installed-usage-guard, usage-stop-notice]
reviewed_revision: "git:a5e5881578cd99f1583a00c5a14bca9bc1324a48"
status: active
---

# 현재 작업의 초기화 감지 전용 제어

- 0.13.0 정식판의 선택형 감시: 실제 초기화 또는 정상 조회 공백 60초 때 한 차례 중단, 15초 시작·5초 제한·2초 재시도
- 재개 잠금·확인 명령 없음. Codex 직접 재개·새 입력 허용, 같은 사건·재시작·시작 전 검사의 중복 중단 억제
- 0.12.1-dev Windows의 이전 네 실행 실제 중단 283~306ms·앱 유지·현재 읽기 4/4. 최종 한 차례 제어의 모의·Rust 회귀와 로컬 설치·정상 감시 확인
- 하한·0% 보호 유지. 다른 OS·클라우드·실제 제공자 초기화·서버 과금 종료 미증명
- 같은 중단·알림 정본 코드와 공개test.1의 세 운영체제 패키지 수용 통과. 새 통합 실제 중단 시험과 구분
