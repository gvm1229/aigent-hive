---
schema_version: 1
pair_id: usage-guard-thresholds
topic_slug: usage-guard-thresholds
language: ko
counterpart: ../en/usage-guard-thresholds.md
title: "사용량 보호 한도"
summary: "전역 설정이 최소 안전 한도를 관리하고 등록 project는 더 이른 중지만 선택 가능."
tags: [guard, project, setup, usage]
aliases: ["조기 중지 한도", "프로젝트 사용량 한도"]
sources:
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:87db5fb3f07e5a346d0060eee545bcd22135963c850afbf0e1fd737ba243b1d1"
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:98b58ba14b69581de4035431ed0a970bd3188e4d8dd63a93e993ebc1d4263c55"
links: [global-onboarding, source-usage-guard, usage-sensor-policy]
reviewed_revision: "git:a5dd671385c2a1e09d511fb1de6c737261210df7"
status: active
---

# 사용량 보호 한도

전역 설정 한도: 사용자 전체의 최소 안전선. 등록 project: 더 높은 남은 사용량 한도만 선택 가능.
실제 한도: `max(global, project)`. 전역 `20%`, 웹 `50%`, 게임 `30%`면 웹은 `50%`, 게임은
`30%`에서 중지. 전역 보호를 끄면 모든 project 보호도 꺼짐. 계획된 이관: 기존 단일 한도를 전역
값으로 보존하고, 잘못되었거나 인증할 수 없는 설정은 쓰기 없이 거부.
