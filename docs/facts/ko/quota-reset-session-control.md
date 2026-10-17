---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: ko
counterpart: ../en/quota-reset-session-control.md
title: "현재 작업의 초기화 감지 전용 제어"
summary: "현재 Windows 단일 중단·재실행 차단 통과, 전체 수용·설치 대기"
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:c18be9c1e87bb1a34d5ffeffa5aab93b2d620b86723a4ccfc05e804103b16aa8"
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:361357e42576b8cbb6f01ee77a6eb0474cc6af4dcb68ddc6d18ede18208bd764"
  - "repo:crates/hive-cli/src/usage_monitor/desktop.ps1#sha256:4b6e8a788b28e12fc34dff7f07cdf03d98814b86b7235dec7e3ea42866deb7e9"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:4ef948e0b4201c4ac5a2864174c58f5d3ba07a3c54eaeafe75c075e4d870a59a"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:c45862bad77bd8cb0b9eb04691cf6161074509e7afeed1effb21cc57b03f532f"
  - "repo:docs/guides/installed-usage-guard.md#sha256:b6df0ad8e439ecfeb01d93e827e4d0f9b54ff8194d52149c41a3869f689ee6e0"
  - "repo:docs/research/session-stop-0.12.1.md#sha256:d5a18bc4b139cd05ff96cd6c9497fe43a15b170c5eeb436523468e9115752eac"
links: [automatic-dispatch-guard, installed-usage-guard]
reviewed_revision: "git:583b3083b9c547fcc385249fe6892c17224fe612"
status: active
---

# 현재 작업의 초기화 감지 전용 제어

- 0.12.1 첫 차수: 실제 초기화 즉시 세션 중단·정상 관측 공백 60초 보호
- 15초 시작·5초 제한·2초 재시도. 평상시 기록 구독 없음, 사건 뒤 5초 확인·구독 해제
- 앱 종료 금지. 공유 사건의 정확한 확인·새 관측 필수, 새 대화·재시작의 잠금 해제와 자동 재개 금지
- Windows 앱 26.1013.4894.0 단일 중단·잠금 중 재실행 중단과 창 유지 통과. 전체 수용·설치 대기, 현재 CLI 0.12.0·전역 감시 비활성
- 기존 하한·0% 보호 유지. 다른 OS·원격·클라우드 미지원, 실제 제공자 초기화·과금 종료 미증명
