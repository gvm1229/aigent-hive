---
schema_version: 1
pair_id: usage-guard-thresholds
topic_slug: usage-guard-thresholds
language: ko
counterpart: ../en/usage-guard-thresholds.md
title: "사용량 보호 한도"
summary: "사용자가 전역 최소 안전 한도를 선택하고 등록 project는 더 이른 중지만 선택 가능."
tags: [guard, project, setup, usage]
aliases: ["조기 중지 한도", "프로젝트 사용량 한도"]
sources:
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:dbd573eccac9845d5112827a43c9fbf0e7538e0b6a186048c9afe041ab491e7e"
links: [global-onboarding, source-usage-guard, usage-sensor-policy]
reviewed_revision: "git:ced55f4d0b18b259c9b43e0f9622b6d617a65737"
status: active
---

# 사용량 보호 한도

- 전역 한도: 사용자 선택
- Project 한도: 더 높은 남은 사용량 값만 허용
- 실제 한도: `max(global, project)`
- 고정 profile 퍼센트: `0건`
- 전역 보호 비활성화: 모든 project 보호도 비활성화
- 이관: 기존 단일 한도를 전역 값으로 보존. 잘못되었거나 인증할 수 없는 설정은 쓰기 없이 거부
- Source 개발: project override 없는 global threshold 사용
- Non-Hive folder: global threshold 상속·project target mutation `0건`
- 현재 maintainer global threshold: remaining `5%`
- source-only guard Skill·adapter·threshold state: `0건`
