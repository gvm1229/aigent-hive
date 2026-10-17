---
schema_version: 1
pair_id: installed-usage-guard
topic_slug: installed-usage-guard
language: ko
counterpart: ../en/installed-usage-guard.md
title: "설치 guard 대상 경계"
summary: "대상별 검사 유지, 현재 Windows 감시의 로컬 설치·활성화"
tags: [guard, source, usage]
aliases: ["Installed usage policy"]
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:fb83162490270dc48650dd60c0a93c321c8811915d863fd102e478f9d22967f4"
  - "repo:docs/guides/installed-usage-guard.md#sha256:369a6681fcc7415c711ab7d5b9d1dae879b9d6fc188261af10a4ffcb5327a6f2"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:da2797509ea7e0ef94783877e3c2cb69c6bfd490"
status: active
---

# 설치 보호의 대상 경계

- 설치 제품의 단일 사용량 보호. 설정한 프로젝트는 전역·프로젝트 하한 중 큰 값
- 소스는 전역 하한·사용자 루트 실행 상태 사용, 소스 내부 소비자 상태 생성 금지
- 미설정 폴더의 대상별 검사·제어·실행 상태 생성 없음, 설정 없이 쓰는 스킬 유지
- 세션 제어는 정확한 설정 대상에 결합. 무관하게 손상된 작업 그래프 CURRENT 보존, 권한 근거 제외
- 소스 작업 시작 전 1회 검사. 별도 소스 감시기·도구마다 반복 검사 금지
- Codex 26.1013.4894.0의 0.12.1 Windows 감시 로컬 설치·활성화. 한 차례 중단·재개 잠금 없음, 무관한 폴더의 소비자 상태 생성 없음
