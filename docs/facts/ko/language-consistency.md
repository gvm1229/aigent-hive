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
  - "repo:.agents/directives/01-behavior.md#sha256:53b809a61225b5d860c37c8c61459960d26306aaf19e550fe79ce50984eebf9e"
  - "repo:.agents/directives/08-human-documentation-style.md#sha256:f1c700565caf1c448cfa0a7d58db549d5c3d466b264737233fe255c67663acd6"
  - "repo:harness/project-bases/0.9.0/AGENTS.md.template#sha256:1aefece59d56d610227b64cfcfff8c634e47202f8e224916b248a8e8ecd9de51"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:f5d9b13356fb64171213e98b41045955760247c6f5e1ce420c991afe450063de"
  - "repo:harness/template/AGENTS.md.jinja#sha256:3d14ecded34d198d08e5aba138239e933fc2670888db4bc3c4637984572076e6"
links: [global-onboarding, source-development]
reviewed_revision: "git:3410f70938d664269f10f39c50028e57498fd248"
status: active
---

# 응답 언어 일관성

개발 에이전트·소비자 하네스: 선택 언어로 모든 질문·응답 작성. 현재
응답의 다른 언어 명시 요청만 예외.

영어 응답: ASD-STE100 Simplified Technical English 적용. 짧고 직접적인 문장, 구체적
동사, 문장당 핵심 하나. 관용 표현·군더더기·모호한 대명사·복잡한 종속절·불필요한 동의어
사용 금지.

한국어 응답: 한국어 어휘·문장 구조 우선. 치환 가능한 영어·혼합 합성어·강조용 영어
괄호 표기 금지. 금지 표현마다 자연스러운 한국어 대체 표현 제시. 완료 기준: 원본·소비자
계약, 사용자 설정 네 투영, 생성된 `AGENTS.md` 수명주기 시험 통과.
