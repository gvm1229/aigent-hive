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
  - "repo:.agents/directives/01-behavior.md#sha256:54f993c4403e7de3612efa12f499bbf1c3a957ee5893875cb0c50204505b1f40"
  - "repo:.agents/directives/08-human-documentation-style.md#sha256:8e045ba8ad7674019e53f7df9ac42b0b8a039af9a5907efc257ea85186e54e0f"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:9cf945c0072f953b590757cffec4fbda47b68ac0145cb4b573b220aa6ddc4d45"
  - "repo:harness/project-bases/0.9.0/AGENTS.md.template#sha256:1aefece59d56d610227b64cfcfff8c634e47202f8e224916b248a8e8ecd9de51"
  - "repo:harness/skills/prompt-refine/SKILL.md#sha256:bbd9a76fed57e1276aa94266709d79f656eafebc98e58723c5f8286b979399ed"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:14b97538abd3ac185627d5dc0a573b540122f780e5549edf80373c6c98866e69"
  - "repo:harness/template/AGENTS.md.jinja#sha256:ab0902d90e75ac32219217b3617c46008f3a906a84a68453bf389fd69e9ec529"
links: [global-onboarding, source-development]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
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
