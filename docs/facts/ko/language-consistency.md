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
  - "repo:.agents/directives/01-behavior.md#sha256:7679dd5603fdfa1104b0017e9fe7c7acb6a81f9e4095233ccddf3db01a325af8"
  - "repo:.agents/directives/08-human-documentation-style.md#sha256:a4f9c9d280a596786fb93cd0ee71bc7b5987f3bafe3be99b1184997f7af6465f"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:633265271a82ee37ef07acd9e5a3d406ea80a3f17c7c9809d3d8d7619dd91260"
  - "repo:harness/project-bases/0.9.0/AGENTS.md.template#sha256:1aefece59d56d610227b64cfcfff8c634e47202f8e224916b248a8e8ecd9de51"
  - "repo:harness/skills/prompt-refine/SKILL.md#sha256:bbd9a76fed57e1276aa94266709d79f656eafebc98e58723c5f8286b979399ed"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:d1105d7d3ebc2c5b8dbf100c4383ecd0933a8425c07501785b84ccdd4c6d2719"
  - "repo:harness/template/AGENTS.md.jinja#sha256:27a80d0856d5f69ed2670eb441068950a42b207d592d9ba892542aa56c99bffc"
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
