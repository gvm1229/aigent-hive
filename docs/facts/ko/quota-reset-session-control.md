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
  - "repo:crates/hive-cli/src/usage_monitor.rs#sha256:7e20a151a1ed78c1aebbaa5c55856ef67dfbdbb765818ced004b46daf706330a"
  - "repo:docs/decisions/reset-stop-boundary.md#sha256:eea2f5a40cb2afa52e7acc6ed070b409d1f28f7b7bde3d545bce07baf9e8ab52"
  - "repo:docs/guides/installed-usage-guard.md#sha256:dc9e92ed1cb85336da05c2f030e20efcdebf15b60751b90777a2307ec8cefef4"
  - "repo:docs/research/session-stop-0.12.1.md#sha256:517fbb1c2f45fca0bdcacb6fa68b3e85ba538a0293dc50fdd8311ec691038d6e"
links: [automatic-dispatch-guard, installed-usage-guard]
reviewed_revision: "git:8f2d7121058024bb4118ae036c6878cc7eae75de"
status: active
---

# 현재 작업의 초기화 감지 전용 제어

- 작업별 제외의 호스트·대화·프로세스 결합, 하한·0% 보호 유지
- Windows 0.12.1의 앱 통로와 작업 중지 처리. 앱 전체 종료 금지, 15초 조회·5초 제한·2초 재시도·30초 공백 중단
- 새 채팅·재시작의 잠금 보존, 정확한 확인·새 관측·제어부 해제 뒤 실행 허용. 자동 재개 금지
- 합성 초기화로 부모·하위 작업 둘의 실제 중단과 앱 유지 확인. 과거 대화 반복 확인의 수용 실패, 수정 뒤 재검증 대기
- 시험·과거 기록·원인·기한 실패 표시. 설치 수용·서버 과금 종료·초과 소비 상한 미증명
