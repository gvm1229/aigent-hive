---
schema_version: 1
pair_id: powershell-hook-exit-status
topic_slug: powershell-hook-exit-status
language: ko
counterpart: ../en/powershell-hook-exit-status.md
title: "PowerShell 훅 종료 상태 경계"
summary: "바깥 PowerShell을 거치며 진단 종료 2가 1로 바뀌는 재현"
tags: [hooks, powershell, verification]
aliases: []
sources:
  - "repo:tests/results/runs/20260920T190500-9123781755ba.md#sha256:31be3f7db845673adcd8b04623aeb9531d60e1bca2b8963afefa18cfd7d9e416"
links: [native-hook-launcher-failure]
reviewed_revision: "git:ff082a7d25848a82d4cf081f1d8e7701a2da618e"
status: active
---

# PowerShell 훅 종료 상태 경계

같은 인코딩 진단 명령을 cmd로 실행하면 종료 2, 바깥 PowerShell로 실행하면 종료 1. 두 경우 표준 오류의 종료 2 진단 문구 일치. 사용자 CLI 실패와 일치하는 직접 재현이며 실제 호스트 셸의 독립 식별과는 별개. 자식 종료 코드의 보존을 가정하지 않고 호스트에 도달한 상태 확인 필요. JSON 거부와 프로세스 실패는 별도 수용 사례.
