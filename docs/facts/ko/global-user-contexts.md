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
  - "repo:README.md#sha256:748b53ba0cfdf7ed37da6ddf80bbf9b9659fd5ed5711b8d43d1f37f669aa4494"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:0ec39fe16f77e7403489393b0ca299c93f8d7dc46830f0e7582a283423b6a03f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:90a8ecca713a1b1963b5f1863f76d32d5c5b9532ca72922c2705ee9b63520307"
  - "repo:harness/user-setup/catalog.yml#sha256:4926655a12591cae061e674d774557e96f000d149f8dec1c2b1b650ba235f494"
  - "repo:schemas/user-setup.schema.json#sha256:57a426a58c822271f1c6297c2c607e532e83c5652ca92ef68bdbcd8b95d357fd"
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
