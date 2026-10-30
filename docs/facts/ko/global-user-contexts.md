---
schema_version: 1
pair_id: global-user-contexts
topic_slug: global-user-contexts
language: ko
counterpart: ../en/global-user-contexts.md
title: "전역 사용자 맥락"
summary: "Global setup의 복수 사용자 맥락: 배경 정보 전용, 한국어 product term 보존."
tags: [bootstrap, communication, onboarding]
aliases: ["사용자 맥락", "사용자 프로필"]
sources:
  - "repo:README.md#sha256:e60a845b60ef813d986cf2a115573c78d20e0b0ec0b6061bdb98a1ed7f76b9c2"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:3c6e0b3a0f62d5598e68024fd7a69d82cb2b0dcfbb29d5602f6f362bc0df2d40"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:828eb911da5e50ef6cadd88c301e07f5aa454c2bbe1c9bcd7ef460f7efae00fe"
  - "repo:harness/user-setup/catalog.yml#sha256:fed2aefc7efa52c28bb05c5b069ad4c4fbeec30b805fff7b84d00285fca18ea4"
  - "repo:schemas/user-setup.schema.json#sha256:4f9c5f59802bc964cfdc41636c32e7e71929c52d64a0291babba948906567a1f"
links: [global-onboarding, language-consistency]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# 전역 사용자 맥락

Global setup: 웹 개발·게임 개발·일반 지식 작업의 복수 조합과 선택형 짧은 설명 저장. 사용자 정보 전용;
project workflow·구현 선택·작업 우선순위·Skill 선택 결정 금지.

모든 built-in Skill 기본 활성화; 필요 시 개별 Skill toggle. Legacy single profile: 같은 context 이관.
Legacy custom profile: 원문 description 보존. Korean setup: `Skill`·`Wiki`·host name product term 유지,
`Skill → 기술` 번역 금지.
