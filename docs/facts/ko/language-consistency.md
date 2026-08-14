---
schema_version: 1
pair_id: language-consistency
topic_slug: language-consistency
language: ko
counterpart: ../en/language-consistency.md
title: "응답 언어 일관성"
summary: "영어 응답은 ASD-STE100, 한국어 응답은 불필요한 영어 혼용 없이 의미 중심으로 작성"
tags: [communication, documentation, harness, language]
aliases: ["언어 일관성", "영어 통제 언어", "한국어 응답"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:6587e8fa5aa274f2c981ad28c062d3c8c388e351440c04663a50122570986976"
  - "repo:.agents/directives/08-human-documentation-style.md#sha256:c5e0f385ab0bdb17979eee241bc77ad8531d5fb4e29198654bb28b9185164884"
  - "repo:harness/project-bases/0.9.0/AGENTS.md.template#sha256:0d3c02bcd6269879b635b83d7ed22a0e4a9fd6e1b15f75a0b2a1e496f808e57d"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:640be28ec7f75444a52544b0d36c45363696dcbd0281f9c5aabd0768d185784e"
  - "repo:harness/template/AGENTS.md.jinja#sha256:63e361ae2218f00f6a22f5e192c25a5c3bcddc21d51f61006f74a0459b636a38"
links: [global-onboarding, source-development]
reviewed_revision: "git:721f888e97222d8c32e67eb5c546dc070189090a"
status: active
---

# 응답 언어 일관성

개발 에이전트·소비자 하네스: 선택 언어로 모든 질문·응답 작성. 현재
응답의 다른 언어 명시 요청만 예외.

영어 응답: ASD-STE100 Simplified Technical English 적용. 짧고 직접적인 문장, 구체적
동사, 문장당 핵심 하나. 관용 표현·군더더기·모호한 대명사·복잡한 종속절·불필요한 동의어
사용 금지.

한국어 응답: 한국어 어휘·문장 구조 사용. 정확한 영어 표기·대체어 없는 용어만 영어
유지. 한글·영어 혼합 합성어와 강조용 영어 괄호 표기 금지. 영어 어순 직역 대신 의미
중심 재작성. 완료 기준: 원본·소비자 계약, 같은 사용자 설정 파일 4개, 생성된
`AGENTS.md` 수명주기 시험 통과.
