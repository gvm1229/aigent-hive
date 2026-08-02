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
  - "repo:.agents/directives/01-behavior.md#sha256:d1e3d4cbc89c962bfae66b5a9c135562bd962fa0d8a3765ad2d150e4a9e41195"
  - "repo:AGENTS.md#sha256:7db000b353cb0510c3519ba9000ef305c490d585c256eb76719c68a4587bb8de"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:9963a01fd3f6e86eee944e3fa331437e283616322c0f15bfb33254e31efff9bd"
  - "repo:harness/template/AGENTS.md.jinja#sha256:c53d41177ef323c50041c8e02928fd1db9904188c22d652d5a80dfbd454228e5"
links: [global-onboarding, source-development]
reviewed_revision: "git:a86bb5bc4aa01c9823fa670e83cb538b9f031cbf"
status: active
---

# 응답 언어 일관성

개발 에이전트와 설치된 소비자 하네스: 모든 질문·응답에 선택 언어 적용. 현재 응답에
다른 언어를 사용하라는 명시적 요청만 예외. 메시지 언어만으로 선호 변경 금지. 사용자
전역·프로젝트 지침에 정확한 `en|ko` 값 투영. 완료 기준: 항상 로드되는 소스 규칙과
단위·정적 계약·연결 수명주기 시험 통과. 요청 배경: 설치 시 선택한 언어의 일관된 적용.
