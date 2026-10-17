---
schema_version: 1
pair_id: installed-usage-guard
topic_slug: installed-usage-guard
language: ko
counterpart: ../en/installed-usage-guard.md
title: "설치 guard 대상 경계"
summary: "대상별 시작 전 검사 유지, 0.12.1 Windows 감시 미검증·비활성"
tags: [guard, source, usage]
aliases: ["Installed usage policy"]
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:c18be9c1e87bb1a34d5ffeffa5aab93b2d620b86723a4ccfc05e804103b16aa8"
  - "repo:docs/guides/installed-usage-guard.md#sha256:b6df0ad8e439ecfeb01d93e827e4d0f9b54ff8194d52149c41a3869f689ee6e0"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:eb62d9db75f946f948b4dada89fa141654e80135"
status: active
---

# 설치 보호의 대상 경계

- 설치 제품의 단일 사용량 보호. 설정한 프로젝트는 전역·프로젝트 하한 중 큰 값
- 소스는 전역 하한·사용자 루트 실행 상태 사용, 소스 내부 소비자 상태 생성 금지
- 미설정 폴더의 대상별 검사·제어·실행 상태 생성 없음, 설정 없이 쓰는 스킬 유지
- 세션 제어는 정확한 설정 대상에 결합. 무관하게 손상된 작업 그래프 CURRENT 보존, 권한 근거 제외
- 소스 작업 시작 전 1회 검사. 별도 소스 감시기·도구마다 반복 검사 금지
- 0.12.1의 폴더 간 Windows 감시 미검증·비활성. 활성화 거부, 무관한 폴더의 소비자 상태 생성 없음
