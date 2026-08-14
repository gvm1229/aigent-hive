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
  - "repo:.agents/directives/01-behavior.md#sha256:42bbd59e702cdce48ac6396d4c5a2f3a9b7574cd99272e22f3279c00b041cba4"
  - "repo:.agents/directives/08-human-documentation-style.md#sha256:f1c700565caf1c448cfa0a7d58db549d5c3d466b264737233fe255c67663acd6"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:b1372a8f956b74320081581b95db7333782e81bec926c2383a6fdc6f1f1dd884"
  - "repo:harness/project-bases/0.9.0/AGENTS.md.template#sha256:1aefece59d56d610227b64cfcfff8c634e47202f8e224916b248a8e8ecd9de51"
  - "repo:harness/skills/prompt-refine/SKILL.md#sha256:13e83b4b2a5a4605f53fb5f12af60dabe961fddf680771024ce300f762541486"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:f5d9b13356fb64171213e98b41045955760247c6f5e1ce420c991afe450063de"
  - "repo:harness/template/AGENTS.md.jinja#sha256:33c0da7ba5156ea1aa0ccc08a8e4f88343cf5f6f896994a7d8b830ac0ad6bb74"
links: [global-onboarding, source-development]
reviewed_revision: "git:3410f70938d664269f10f39c50028e57498fd248"
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
