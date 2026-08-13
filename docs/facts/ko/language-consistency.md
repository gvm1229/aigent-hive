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
  - "repo:.agents/directives/01-behavior.md#sha256:9d8adb7c75015fd24df8cb226a16180548c600dc963ee154c0a4af408e9fa48c"
  - "repo:AGENTS.md#sha256:e200aeee1a58fe9224e6e7a0a5a915cde8ac666e4421ea9c6696f0f1cb142fee"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:db91b9841c057a3f9b964185fb2a2f3c2f8701908cf6439e26bf05d389a7243d"
  - "repo:harness/template/AGENTS.md.jinja#sha256:9b45162b48b5d6eedcce0d0b6bc2d63613776fc1089a7d2878941897d0d506c5"
links: [global-onboarding, source-development]
reviewed_revision: "git:35e6b79a024350487f823780101a28be24a9f4c7"
status: active
---

# 응답 언어 일관성

개발 에이전트와 설치된 소비자 하네스: 모든 질문·응답에 선택 언어 적용. 현재 응답에
다른 언어를 사용하라는 명시적 요청만 예외. 메시지 언어만으로 선호 변경 금지. 사용자
전역·프로젝트 지침에 정확한 `en|ko` 값 투영. 완료 기준: 항상 로드되는 소스 규칙과
단위·정적 계약·연결 수명주기 시험 통과. 요청 배경: 설치 시 선택한 언어의 일관된 적용.
