---
schema_version: 1
pair_id: verification-result-clarity
topic_slug: verification-result-clarity
language: ko
counterpart: ../en/verification-result-clarity.md
title: "검증 결과 명확성"
summary: "Hive 검증 결과에 실제 실행 여부, 미실행 이유, 입증 범위와 미검증 범위를 명시."
tags: [communication, reporting, verification]
aliases: ["건너뜀 보고", "검증 한정 조건"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:94e9c3723758b46a23989ca9264b0951c4e9bf1096c7c5766d39697bf501f7e3"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:d16fe3cf14d23b5781bbb960884146b17c3770916a5572215e4922a4423f28d1"
  - "repo:docs/guidance-schema.md#sha256:eae385d284f448a27a5243d8e7846aa69d9568e0849d3457147fb814229416ad"
  - "repo:harness/template/AGENTS.md.jinja#sha256:02cfaea5d05fc9e51b5a368cc70290f6f2b041912433bf640ad2ccaa0f6b2c39"
links: [language-consistency, release-verification]
reviewed_revision: "git:ccf59c00e326034d6d12ef9a9c8a52c91d5ff8cc"
status: active
---

# 검증 결과 명확성

Hive 개발 에이전트와 설치된 소비자 하네스: 통과·실패·건너뜀·연기·미검증·미지원
결과마다 영향 범위, 정확한 이유, 현재 실행 환경과의 관계, 실제 실행 여부, 결과가
입증하는 것과 미검증으로 남기는 것을 함께 보고. 운영체제 이름만으로 실행 여부를
대신 표현 금지. 완료 기준: 개발·소비자·프로젝트 지침 일치와 투영 시험 통과.
요청 배경: Windows 검사 결과가 사전 검증인지 운영체제 오인인지 구분되지 않았던
모호한 보고의 재발 방지.
