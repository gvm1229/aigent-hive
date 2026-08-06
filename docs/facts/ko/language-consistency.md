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
  - "repo:.agents/directives/01-behavior.md#sha256:b84d45bd3fb28b67581ff024fdfb8d0aaf0dfbbda580f712c96d1228685dadbb"
  - "repo:AGENTS.md#sha256:5a870d5e7350ee330c5ac861ec306f2e309b63974da34ac7e0e28594ec744760"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:35f3cace4b6297a298b8b59db208b3d8ecfd82331758fb6bd34dd1ec03aa8ec7"
  - "repo:harness/template/AGENTS.md.jinja#sha256:070d97440343d699565448c239efb55c905df79119df289525d41edc6e81581f"
links: [global-onboarding, source-development]
reviewed_revision: "git:35e6b79a024350487f823780101a28be24a9f4c7"
status: active
---

# 응답 언어 일관성

개발 에이전트와 설치된 소비자 하네스: 모든 질문·응답에 선택 언어 적용. 현재 응답에
다른 언어를 사용하라는 명시적 요청만 예외. 메시지 언어만으로 선호 변경 금지. 사용자
전역·프로젝트 지침에 정확한 `en|ko` 값 투영. 완료 기준: 항상 로드되는 소스 규칙과
단위·정적 계약·연결 수명주기 시험 통과. 요청 배경: 설치 시 선택한 언어의 일관된 적용.
