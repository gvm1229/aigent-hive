---
schema_version: 1
pair_id: windows-monitor-module-deadline
topic_slug: windows-monitor-module-deadline
language: ko
counterpart: ../en/windows-monitor-module-deadline.md
title: "Windows 감시의 모듈 시작 기한"
summary: "고정 모듈 경로로 초기 조회 기한 보존"
tags: [isolation, powershell, usage-guard, windows]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_monitor/windows.ps1#sha256:d3908b27c3e339a7ff99d2ebd6008c27055abbb26f3f43cfc04b49b9e0406b13"
  - "repo:tests/results/runs/20261028T094859-a3ae3ce2a902.md#sha256:36254ee6ccc20f7f990f9d97a161657a9c7d0b6a4d11ef70e8e19850ae034a89"
  - "repo:tests/results/runs/20261028T095333-28c178240686.md#sha256:5c11e35200273275f2c5e814ff418a20e7a0b0bf550916ab14fd79ee3affb34c"
links: [windows-powershell-module-isolation]
reviewed_revision: "git:5e4c55012c03f15ccc0f8cad2aaa8f3403871b1c"
status: active
---

# Windows 감시의 모듈 시작 기한

0.14.0 감시 시작 실패 조사: 격리 Windows Server2025에서 자동 모듈 검색28.703초·고정 경로 사본0.875초 확인. 고정 스크립트의 PSHOME Utility/CimCmdlets 명시 로드, 조회에 Appx·예약 처리에 ScheduledTasks 선택. 요청 기한·소유 검사 유지. 읽기 전용 진단과 예약 대역 검사 범위, 전체 활성화·실제 앱 제어·공개 출시 수용과 구분.
