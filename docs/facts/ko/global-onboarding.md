---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: ko
counterpart: ../en/global-onboarding.md
title: "Global onboarding"
summary: "복수 사용자 맥락 저장과 project workflow 비결정, authenticated Hive-only refresh 자동 처리, 한국어 product term·all built-in Skill 기본값 유지."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:30e7d1dece221c145e4a75fe9e05ec9520ca3ab58b7d1311088b9c4ad72759ef"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:a4d69016bdf3c5f0e8ee75839c7076b804ce55a8583fa56aabd933545d148611"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:03fad2cb09cd32e0f9ecc6c586a1f088fbfec2d0a01094af320dc4cf4d9200d5"
  - "repo:harness/skills/setup-hive/SKILL.md#sha256:2851a369d75eaa79fe50ba9295787c09edbf7b25163e6fb64260ceba472db843"
  - "repo:harness/user-setup/catalog.yml#sha256:af1147b8468f48eb81ec77ed4a14d5eba2fd31a4302e5459544fec3b2e22b595"
  - "repo:schemas/user-setup.schema.json#sha256:b94594a2597f8eab3bcb778c24b892ee45c3856ce421043a79c3861b59cb99ee"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:0c0a3fd18bd4b3746202c5a38aa7cb03d4b94908"
status: active
---

# Global onboarding

수동 첫 설정 순서: CLI 설치, host activation, global setup, 명시 project setup. 선택형 one-prompt 경로:
project inspection 없는 global setup 시작.

지원 legacy 복구: saved preference·live file evidence 일치 조건. 그 외 active byte 보존. 명시 global
setup 요청: authenticated Hive-only install 또는 saved-answer user projection refresh의 preview·apply·
revalidate 자동 처리, review-only 질문 없음. 사용자 맥락, Skill 선택, 한국어 product term:
`global-user-contexts` 참고.
