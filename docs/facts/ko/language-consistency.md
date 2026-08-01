---
schema_version: 1
pair_id: language-consistency
topic_slug: language-consistency
language: ko
counterpart: ../en/language-consistency.md
title: "응답 언어 일관성"
summary: "현재 응답에 다른 언어를 사용하라는 명시적 요청이 없는 한 개발 에이전트와 소비자 하네스의 질문·응답을 선택 언어로 통일."
tags: [communication, documentation, projection]
aliases: ["언어 일관성", "한영 혼용 방지"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:69cad89a5e857e404f6d51106a8688623afd6d3ad1613ddc5a326ab7b998bb30"
  - "repo:AGENTS.md#sha256:14a0d85c5435cebe820cfd9d8fd1271d1fdce73b0ee878f818350b3e1c619fbd"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:c3cc02dcd02afddbd583d51bd02bc113dc283a17e8244587e0bbf832450dd823"
  - "repo:harness/template/AGENTS.md.jinja#sha256:71eeaf7aff5e21b8a7cf764daf6060cb44954f14218370585c3d72a6f25f14c7"
links: [global-onboarding, source-development]
reviewed_revision: "git:19eda4d7ef87fe3122c14c455df07758c3dc6ff1"
status: active
---

# 응답 언어 일관성

개발 에이전트와 설치된 소비자 하네스: 모든 질문·응답에 선택 언어 적용. 현재 응답에
다른 언어를 사용하라는 명시적 요청만 예외. 메시지 언어만으로 선호 변경 금지. 사용자
전역·프로젝트 지침에 정확한 `en|ko` 값 투영. 완료 기준: 항상 로드되는 소스 규칙과
단위·정적 계약·연결 수명주기 시험 통과. 요청 배경: 설치 시 선택한 언어의 일관된 적용.
