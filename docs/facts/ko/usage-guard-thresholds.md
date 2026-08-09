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
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:2150681617bd1c2273780f0796609f27fc4815418428c0743ef11b88245deb38"
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:fed21b2de4b06f8034974ea611ce0afb2c0b09244a57c16238a2a1c662a131f8"
links: [global-onboarding, source-usage-guard, usage-sensor-policy]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
status: active
---

# 사용량 보호 한도

- 전역 한도: 사용자 선택
- Project 한도: 더 높은 남은 사용량 값만 허용
- 실제 한도: `max(global, project)`
- 고정 profile 퍼센트: `0건`
- 전역 보호 비활성화: 모든 project 보호도 비활성화
- 이관: 기존 단일 한도를 전역 값으로 보존. 잘못되었거나 인증할 수 없는 설정은 쓰기 없이 거부
- Source 개발: 같은 product guard·resolver·project override 사용
- 이관 완료 뒤 source-only guard Skill·adapter·threshold state: `0건`
