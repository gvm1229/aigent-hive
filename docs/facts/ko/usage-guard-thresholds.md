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
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:5a60d254c760db58049da72530895a981708d549700b02656c7ff51224140f5f"
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:28bc5662cca5ecb730361d7e6519890c0f9db2f800720e9c75d90a854e3d0c80"
links: [global-onboarding, source-usage-guard, usage-sensor-policy]
reviewed_revision: "git:6ed32f63fa3c67bed31164b9d15259f48443341a"
status: active
---

# 사용량 보호 한도

전역 설정에서 사용자 전체의 최소 안전 한도 선택. 등록 project: 더 높은 남은 사용량 한도만 선택.
실제 한도: `max(global, project)`. project profile·문서의 고정 퍼센트 없음. 전역 보호를 끄면
모든 project 보호도 꺼짐. 계획된 이관: 기존 단일 한도를 전역 값으로 보존하고, 잘못되었거나
인증할 수 없는 설정은 쓰기 없이 거부.
