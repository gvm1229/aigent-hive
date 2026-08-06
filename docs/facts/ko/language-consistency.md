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
  - "repo:.agents/directives/01-behavior.md#sha256:24e61b7fd37bc1b9e0a73933547d5b369b9ca2cdde6c9adc10ba29bd23d50143"
  - "repo:AGENTS.md#sha256:8a5f2a661c03a43976d7e88bce188a07a8e17db569b82ae76e83c3807914b30a"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:39bc19a47799793c2f2e984f5d7d6edb4e18fbbd96ec33ac30e7c258fda66d0b"
  - "repo:harness/template/AGENTS.md.jinja#sha256:070d97440343d699565448c239efb55c905df79119df289525d41edc6e81581f"
links: [global-onboarding, source-development]
reviewed_revision: "git:33f365d3dbb1af51333a6dbb1834ce437a932ea0"
status: active
---

# 응답 언어 일관성

개발 에이전트와 설치된 소비자 하네스: 모든 질문·응답에 선택 언어 적용. 현재 응답에
다른 언어를 사용하라는 명시적 요청만 예외. 메시지 언어만으로 선호 변경 금지. 사용자
전역·프로젝트 지침에 정확한 `en|ko` 값 투영. 완료 기준: 항상 로드되는 소스 규칙과
단위·정적 계약·연결 수명주기 시험 통과. 요청 배경: 설치 시 선택한 언어의 일관된 적용.
