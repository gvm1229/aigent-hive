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
  - "repo:.agents/directives/01-behavior.md#sha256:69cad89a5e857e404f6d51106a8688623afd6d3ad1613ddc5a326ab7b998bb30"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:c3cc02dcd02afddbd583d51bd02bc113dc283a17e8244587e0bbf832450dd823"
  - "repo:docs/guidance-schema.md#sha256:99c034ed85314fa0f707f057e4e567cfb32159e9bd50e5f81388c37de740c2e6"
  - "repo:harness/template/AGENTS.md.jinja#sha256:71eeaf7aff5e21b8a7cf764daf6060cb44954f14218370585c3d72a6f25f14c7"
links: [language-consistency, release-verification]
reviewed_revision: "git:19eda4d7ef87fe3122c14c455df07758c3dc6ff1"
status: active
---

# 검증 결과 명확성

Hive 개발 에이전트와 설치된 소비자 하네스: 통과·실패·건너뜀·연기·미검증·미지원
결과마다 영향 범위, 정확한 이유, 현재 실행 환경과의 관계, 실제 실행 여부, 결과가
입증하는 것과 미검증으로 남기는 것을 함께 보고. 운영체제 이름만으로 실행 여부를
대신 표현 금지. 완료 기준: 개발·소비자·프로젝트 지침 일치와 투영 시험 통과.
요청 배경: Windows 검사 결과가 사전 검증인지 운영체제 오인인지 구분되지 않았던
모호한 보고의 재발 방지.
