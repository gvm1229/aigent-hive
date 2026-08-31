---
schema_version: 1
pair_id: language-consistency
topic_slug: language-consistency
language: ko
counterpart: ../en/language-consistency.md
title: "응답 언어 일관성"
summary: "영어 응답은 ASD-STE100, 한국어 응답은 불필요한 영어 혼용 없이 의미 중심으로 작성. Hive 프롬프트는 현재 언어 명시가 없으면 영어 기본값"
tags: [communication, documentation, harness, language]
aliases: ["언어 일관성", "영어 통제 언어", "한국어 응답"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:3a8450ff3e496f4e6bafc7b8d10cdd9fe38f15932b465d131a69ca0bdf9ef2f3"
  - "repo:.agents/directives/08-human-documentation-style.md#sha256:f1c700565caf1c448cfa0a7d58db549d5c3d466b264737233fe255c67663acd6"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:acd4022de5697806003207634ac0b7cb874baeb802af491f28d39ec048daf830"
  - "repo:harness/project-bases/0.9.0/AGENTS.md.template#sha256:1aefece59d56d610227b64cfcfff8c634e47202f8e224916b248a8e8ecd9de51"
  - "repo:harness/skills/prompt-refine/SKILL.md#sha256:3848758e0725a7b9b990d3055f22942ec6aededee7d3c8255d0162c8633c6fc5"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:914cca3de8883e2b1be0dfbea92da3dd2c856cdca53ed24d3bd45d9ff75b6cd2"
  - "repo:harness/template/AGENTS.md.jinja#sha256:7fdcd351b7d0624baa68d11bf9e850692c9eeaae13abb003295c8727f621543a"
links: [global-onboarding, source-development]
reviewed_revision: "git:64125db02505a9a696e870d23fa54feb125b8093"
status: active
---

# 응답 언어 일관성

개발 에이전트·소비자 하네스: 선택 응답 언어로 질문·응답 작성. 현재 응답의 다른 언어
명시 요청만 예외.

Hive 작성·개선·복사용 프롬프트: 응답 언어와 분리. 현재 프롬프트 언어의 명시 요청이
없으면 영어 기본값. 명시 언어 우선. 주변 설명·질문은 선택 응답 언어 유지.

영어 응답·기본 프롬프트: ASD-STE100 Simplified Technical English. 한국어 응답: 한국어
어휘·문장 구조 우선. 치환 가능한 영어·혼합 합성어·강조용 영어 괄호 표기 금지. 원본·
소비자 계약의 금지·대체 예시와 투영 시험 기준 적용.
