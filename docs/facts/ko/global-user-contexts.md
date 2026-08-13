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
  - "repo:README.md#sha256:206f1001bd6d97ce6de5342afc628c9256e84b11439d55b8b78bb3322d219979"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:9fa9e439ad15ea6a8b5ed7cf6d031595a8979b056dada55360cb32331d9e8355"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:de84c29ec9221b5b0bd531e31f21d5317424ed8bdb3f1cfe019670d7f2e876c4"
  - "repo:harness/user-setup/catalog.yml#sha256:3f24914859e7bcbe9bb8c85aafeee4250bdc2da383d0480d000a967fcb3305c5"
  - "repo:schemas/user-setup.schema.json#sha256:83427614c5b997a695b9f22c52093d4e2d26892b7eb42fc9873309891d0e81e0"
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
