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
  - "repo:README.md#sha256:a03aae178a8c1060d3f4301d4ed592a24e8cf9e9e95a7b87afa434804ad4ecbb"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:dc14d336b41cdb4e717abdfab7dd47c921e09618616d5d5e2559ea1b166f4964"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:9ffa22cf14504ba7385135c1f62fdcb19bede32a0925ed72eb23fa8b96359eb5"
  - "repo:harness/user-setup/catalog.yml#sha256:4926655a12591cae061e674d774557e96f000d149f8dec1c2b1b650ba235f494"
  - "repo:schemas/user-setup.schema.json#sha256:e83e5f318a5b6ffcc08cfe0898a2b6138512c6bfb0eea99c6070b134f3712f47"
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
