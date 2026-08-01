---
schema_version: 1
pair_id: language-consistency
topic_slug: language-consistency
language: ko
counterpart: ../en/language-consistency.md
title: "응답 언어 일관성"
summary: "개발 에이전트와 소비자 하네스의 질문·응답을 선택 언어로 통일."
tags: [communication, documentation, projection]
aliases: ["언어 일관성", "한영 혼용 방지"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:a78fc02202dc5c3b934e28924dd86660d297151f4905606dc7a26f2179083eaa"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:e21faccc9dae23d7522de433e345890509ce8d742fa8fe6a375f0892e35713db"
  - "repo:harness/template/AGENTS.md.jinja#sha256:9e5694a62099d262872bd6e1f167d839d9eb3f51c3d6cdfd4884656350cc0ec4"
links: [global-onboarding, source-development]
reviewed_revision: "git:bd6d9249b8641590269d32deb97d13b2816ba75e"
status: active
---

# 응답 언어 일관성

개발 에이전트와 설치된 소비자 하네스: 질문·응답 전체에 선택 언어 적용.
한국어에서는 고유명사, 제품·패키지 이름, 명령어, 코드 식별자, 경로, 스키마 키,
정확한 화면 문구, 뚜렷한 한국어 대체어가 없는 용어만 영어 유지. 영어 문단은
정확한 한국어 이름·문자열·인용문·사용자 보존 요청을 제외하고 영어로 통일.
완료 기준: 개발 지침·사용자 전역 지침·프로젝트 지침 일치와 투영·언어 회귀 시험 통과.
요청 배경: 실행 항목의 이해를 방해한 불필요한 한영 혼용 방지.
