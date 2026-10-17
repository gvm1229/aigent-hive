---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: ko
counterpart: ../en/quota-reset-session-control.md
title: "현재 작업의 초기화 감지 전용 제어"
summary: "네 실행의 실제 중단과 읽기 전용 4/4 확인, 설치 대기"
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:c18be9c1e87bb1a34d5ffeffa5aab93b2d620b86723a4ccfc05e804103b16aa8"
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:361357e42576b8cbb6f01ee77a6eb0474cc6af4dcb68ddc6d18ede18208bd764"
  - "repo:crates/hive-cli/src/usage_monitor/desktop.ps1#sha256:35d95c3c6c1283d972303f95361633bad0c8bc1889493bd00e6d8925e80d35d1"
  - "repo:crates/hive-cli/src/usage_monitor/state.rs#sha256:4ef948e0b4201c4ac5a2864174c58f5d3ba07a3c54eaeafe75c075e4d870a59a"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:c45862bad77bd8cb0b9eb04691cf6161074509e7afeed1effb21cc57b03f532f"
  - "repo:docs/guides/installed-usage-guard.md#sha256:b6df0ad8e439ecfeb01d93e827e4d0f9b54ff8194d52149c41a3869f689ee6e0"
  - "repo:docs/research/session-stop-0.12.1.md#sha256:d003fffb096de7ec2e1f009d0b724ec77c63d0ee1013ad0a972b614fe2937a11"
  - "repo:docs/research/usage-stop-notice-0.12.1.md#sha256:c8b0b2678355ce54e64eed1d2e0100049c54bd59548947f6f1eb37d637455568"
  - "repo:tests/results/session-stop-partial-windows-0.12.1.json#sha256:1f1d27dae9ef95f345c39fa018dce9f01b752cd765d24de3609b1ac7280a4071"
links: [automatic-dispatch-guard, installed-usage-guard, usage-stop-notice]
reviewed_revision: "git:d6c851d2dee4cea3e4d48c6f30c96d24f8c678c0"
status: active
---

# 현재 작업의 초기화 감지 전용 제어

- 0.12.1 첫 차수: 실제 초기화 즉시 세션 중단·정상 관측 공백 60초 보호. 15초 시작·5초 제한·2초 재시도
- 앱 종료 금지. 공유 사건의 정확한 확인·새 관측 필수, 새 대화·재시작의 잠금 해제와 자동 재개 금지
- Windows 26.1013.4894.0의 네 실행 중단 283~306ms·앱 유지. 수정한 확인의 실제 읽기 4/4·1180ms
- 원래 전체 자동 확인 실패 보존, 동일 중단 코드·현재 읽기·표시·센서 실패·명시 확인의 결합 수용. 설치 대기
- 하한·0% 보호 유지. 다른 OS·원격·클라우드 미지원, 실제 제공자 초기화·서버 과금 종료 미증명
