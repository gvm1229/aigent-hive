---
schema_version: 1
pair_id: installed-usage-guard
topic_slug: installed-usage-guard
language: ko
counterpart: ../en/installed-usage-guard.md
title: "설치 guard 대상 경계"
summary: "설정 대상 경계·앱 유지·잠금 없는 재개와 감시 상태 구분"
tags: [guard, source, usage]
aliases: ["Installed usage policy"]
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:1545e670dfd7be94e0d46f9e019d18c4489c2dfebb3c5e5ae7a3fd69d2b5b074"
  - "repo:docs/guides/installed-usage-guard.md#sha256:26f34a3f463b6cb2cf55da617e2c948406b73101dbe7d61e6a201e462f13a0cd"
links: [automatic-dispatch-guard, source-development, usage-guard-thresholds]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# 설치 보호의 대상 경계

- 설정 프로젝트는 전역·프로젝트 최대 하한, 소스는 전역 정책·사용자 루트 상태. 소스 내부 소비자 상태 금지
- 미설정 폴더의 대상별 제어·상태 없음, 설정 없이 쓰는 스킬 유지
- 세션 제어는 정확한 설정 대상에 결합. 손상된 별도 그래프 CURRENT는 권한 근거 제외
- 소스 시작 전 한 번 검사, 별도 감시기·도구마다 반복 검사 금지
- 선택형 Windows 감시는 앱 유지·잠금 없는 직접 재개. 활성 설정과 실제 감시 생존 구분, 프로젝트 밖 전역 감시 해제 허용
