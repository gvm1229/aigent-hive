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
  - "repo:README.md#sha256:413ed120770591773c5efab11aa1bc3587687b411eff47a665802b5bf0f5ea2b"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:6de1cf5f473fc0c6e61504b07ac8eb892abb77231b406d7952dc271e0ee23c1b"
  - "repo:harness/skills/configure/SKILL.md#sha256:17a80a35d5f367421c661374dec54147d0cabb4f48c4c5a640b15253bd5f0222"
  - "repo:harness/user-setup/catalog.yml#sha256:7dc82dbf559075ce4286e7dd19aec0ddc22e04f35ad4a8a60f43129a4dba2a1f"
  - "repo:schemas/user-setup.schema.json#sha256:87bb452a4240faccdef5c96488b7492c3764f44a2819e8e7733b8c41dadc70b9"
links: [global-onboarding, language-consistency]
reviewed_revision: "git:a679bb4d1ea439ef172e8a7f59b649d6d34a1983"
status: active
---

# 전역 사용자 맥락

Global setup: 웹 개발·게임 개발·일반 지식 작업의 복수 조합과 선택형 짧은 설명 저장. 사용자 정보 전용;
project workflow·구현 선택·작업 우선순위·Skill 선택 결정 금지.

모든 built-in Skill 기본 활성화; 필요 시 개별 Skill toggle. Legacy single profile: 같은 context 이관.
Legacy custom profile: 원문 description 보존. Korean setup: `Skill`·`Wiki`·host name product term 유지,
`Skill → 기술` 번역 금지.
