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
  - "repo:README.md#sha256:362f1c802d9f436ffc33682d07709ed9655ce8fa098085f8d930fba93a84888e"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:786da31401085e9445495aa37defe7cedf781bc8457211a6addd23016c0bf922"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:1fcbb2b9b2db6d57bd40682f80db2a0a916ebbffb3434431038b609b6b743c11"
  - "repo:harness/user-setup/catalog.yml#sha256:c256f7c6e33eb8f32530d0a64b5992437445eb058e52760d8cc5f9105e971436"
  - "repo:schemas/user-setup.schema.json#sha256:2c71672d4828b6ccd230165757356b95a75fc5bbb982e28fb57fb0e1e7c12c56"
links: [global-onboarding, language-consistency]
reviewed_revision: "git:01df1d580d987e7fb0f34978076cd000263fd99f"
status: active
---

# 전역 사용자 맥락

Global setup: 웹 개발·게임 개발·일반 지식 작업의 복수 조합과 선택형 짧은 설명 저장. 사용자 정보 전용;
project workflow·구현 선택·작업 우선순위·Skill 선택 결정 금지.

모든 built-in Skill 기본 활성화; 필요 시 개별 Skill toggle. Legacy single profile: 같은 context 이관.
Legacy custom profile: 원문 description 보존. Korean setup: `Skill`·`Wiki`·host name product term 유지,
`Skill → 기술` 번역 금지.
