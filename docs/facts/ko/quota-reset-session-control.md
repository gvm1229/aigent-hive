---
schema_version: 1
pair_id: quota-reset-session-control
topic_slug: quota-reset-session-control
language: ko
counterpart: ../en/quota-reset-session-control.md
title: "현재 작업의 초기화 감지 전용 제어"
summary: "앱 유지·세션 중단·원인 표시와 정확한 사건 확인의 경계"
tags: [reset, session, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:c18be9c1e87bb1a34d5ffeffa5aab93b2d620b86723a4ccfc05e804103b16aa8"
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:33b5733343ccefcf6d212a98729884f1952e1ab87a37a5715ed20f4197dcf39c"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:eea2f5a40cb2afa52e7acc6ed070b409d1f28f7b7bde3d545bce07baf9e8ab52"
  - "repo:docs/guides/installed-usage-guard.md#sha256:dc9e92ed1cb85336da05c2f030e20efcdebf15b60751b90777a2307ec8cefef4"
links: [automatic-dispatch-guard, installed-usage-guard]
reviewed_revision: "git:173c8739c9e593238cfc0b55cbbe52b0033d53dc"
status: active
---

# 현재 작업의 초기화 감지 전용 제어

- 작업별 제외의 호스트·대화·프로세스 결합, 하한·0% 보호 유지
- 0.12.1 Windows: 검증한 앱 통로의 작업 중지 처리, 앱 전체 종료 금지. 15초 조회·5초 제한·2초 재시도·30초 정상 관측 공백 중단
- 새 채팅·재시작에도 사건 잠금 보존. 정확한 확인·새 관측·제어부 중단 해제 뒤 실행 허용, 자동 재개 금지
- 상태 창의 원인·실제 중단·앱 생존·실패 구분. 현재 앱 읽기와 회귀는 실제 중단·표시·설치 수용의 대체 증명에서 제외
