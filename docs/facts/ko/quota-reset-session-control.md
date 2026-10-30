---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: ko
counterpart: ../en/quota-reset-session-control.md
title: "현재 작업의 초기화 감지 전용 제어"
summary: "0.13.1 구현의 재시작·만료 이전값 비교 제외, 하한·0% 보호 유지"
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:9b858bef8831bed659b0f977ecd9ba84111800176d9dfc048b1d46c4c6ab8115"
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:ad6bc77e2e15a0cb969c7066929d8840d90ad6fbce309f526f43100ba31a52c2"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:6effcddaa5061365989f4ce449664b30f59bebc1eaf15f6fdf08f2d9e741a102"
  - "repo:crates/hive-core/src/usage_guard.rs#sha256:e59ef8b5bfe8b2024bb68a6940e23bf9ea45a4af611d7f5d2078f11c1cdfe228"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:212411004c3dcfe328442a665fa411d7ab2a942baa0b654aacc38b585c54d55a"
  - "repo:docs/guides/installed-usage-guard.md#sha256:0ead68f33595fcbd18d4d9c9ec38926645ca3cf22ffd895a02628c5e3c97a384"
  - "repo:docs/research/public-test-0.13.0.md#sha256:c831b239c282a19ee42a0cbf9779484f044bd973425d61f319e9c9947de7a2f9"
  - "repo:docs/research/release-0.13.0.md#sha256:fbdd1a5c8e306a8af4e54cc12306d9da35a41ba71ea7493adbc4de424be8f1a2"
  - "repo:docs/research/session-stop-0.12.1.md#sha256:50b5ba7baa1b9c2ed7bf96669115860655bb2fe0b7b5d7df7ac5fa628c650437"
  - "repo:docs/research/stable-release-0.13.0.md#sha256:75d8a1e6acf84fcb526d57d93166edfcfb52964591cf207fa733fe6ff3fa1b4b"
  - "repo:docs/research/usage-preflight-baseline-0.13.1.md#sha256:28e926a63e9f99f22b8c1331956d0bdb12959af37d29c867cb7234e1ed534761"
  - "repo:docs/research/usage-stop-notice-0.12.1.md#sha256:58148052d858dec443c5d8231c0ad91c7748dfd8e54e2a5b5939e6fc5790cffd"
  - "repo:tests/results/session-stop-one-shot-code-0.12.1.json#sha256:f97ceca1df94fff473e56240f8db49dce4ffe644b455154439008e67a3c21b56"
  - "repo:tests/results/session-stop-one-shot-installation-0.12.1.json#sha256:bbc233760b14fc02ebf35467618df2e59c26bab4e549806634a3c96d058d2371"
  - "repo:tests/results/session-stop-one-shot-qualification-0.12.1.json#sha256:000da296ad912dc0d30d30baed0f79f8a88193d6b09606bd75eff29e1dbe4406"
  - "repo:tests/results/session-stop-partial-windows-0.12.1.json#sha256:1f1d27dae9ef95f345c39fa018dce9f01b752cd765d24de3609b1ac7280a4071"
  - "repo:tests/results/usage-preflight-baseline-0.13.1.json#sha256:863442407f17d14fe0f89fd5c296ca8b7214361b3916b10820320df12b164c7e"
links: [automatic-dispatch-guard, installed-usage-guard, usage-stop-notice]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# 현재 작업의 초기화 감지 전용 제어

- 0.13.0 정식판의 선택형 감시: 실제 초기화 또는 정상 조회 공백 60초 때 한 차례 중단, 15초 시작·5초 제한·2초 재시도
- 재개 잠금·확인 명령 없음. Codex 직접 재개·새 입력 허용, 같은 사건·재시작·시작 전 검사의 중복 중단 억제
- 0.12.1-dev Windows의 이전 네 실행 실제 중단 283~306ms·앱 유지·현재 읽기 4/4. 최종 한 차례 제어의 모의·Rust 회귀와 로컬 설치·정상 감시 확인
- 하한·0% 보호 유지. 다른 OS·클라우드·실제 제공자 초기화·서버 과금 종료 미증명
- 같은 중단·알림 정본 코드와 공개test.1의 세 운영체제 패키지 수용 통과. 새 통합 실제 중단 시험과 구분
