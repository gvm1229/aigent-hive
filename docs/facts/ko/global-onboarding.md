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
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:6de1cf5f473fc0c6e61504b07ac8eb892abb77231b406d7952dc271e0ee23c1b"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:be6e9fd0b94f9cf8a994cce4bb1e8f5b0e8396420968832e285de366dc8e16f9"
  - "repo:harness/skills/configure/SKILL.md#sha256:17a80a35d5f367421c661374dec54147d0cabb4f48c4c5a640b15253bd5f0222"
  - "repo:harness/user-setup/catalog.yml#sha256:7dc82dbf559075ce4286e7dd19aec0ddc22e04f35ad4a8a60f43129a4dba2a1f"
  - "repo:schemas/user-setup.schema.json#sha256:87bb452a4240faccdef5c96488b7492c3764f44a2819e8e7733b8c41dadc70b9"
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
