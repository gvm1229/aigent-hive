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
  - "repo:README.md#sha256:27679c3c338ef2f82b352800ccb882c2536bcc2c7dbfd18b93df52e3349554b0"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:2a7c02ca89bc80f95574e9c6147af3d634bcfd3c40f395a554a225175bf09d91"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:a6aea1ed5b977bc818bace5c9d712d2da01328f59753e9b93136c17b1a8f24d3"
  - "repo:harness/user-setup/catalog.yml#sha256:fed2aefc7efa52c28bb05c5b069ad4c4fbeec30b805fff7b84d00285fca18ea4"
  - "repo:schemas/user-setup.schema.json#sha256:daee52c6535601606bc39d67800ed2e6ad248828ac73383cc7d8ded015c95652"
links: [global-onboarding, language-consistency]
reviewed_revision: "git:1b755a995d91739d758830210d93cdc012e9e61b"
status: active
---

# 전역 사용자 맥락

Global setup: 웹 개발·게임 개발·일반 지식 작업의 복수 조합과 선택형 짧은 설명 저장. 사용자 정보 전용;
project workflow·구현 선택·작업 우선순위·Skill 선택 결정 금지.

모든 built-in Skill 기본 활성화; 필요 시 개별 Skill toggle. Legacy single profile: 같은 context 이관.
Legacy custom profile: 원문 description 보존. Korean setup: `Skill`·`Wiki`·host name product term 유지,
`Skill → 기술` 번역 금지.
