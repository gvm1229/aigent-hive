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
  - "repo:.agents/directives/01-behavior.md#sha256:2532c785b59f23a099b9e4a6eb71798f696dc4b79103600cf7c245582afa9f26"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:c5cb31b7cf39c02be926e38ee529e023aabe45870b84a75b711f4f84c424e282"
  - "repo:docs/guidance-schema.md#sha256:881e80dda2f29dbc6239177aa66f518eff5bfb67dd93db017d6c280d914a7452"
  - "repo:harness/template/AGENTS.md.jinja#sha256:e9545c960f609ad7369e2d5e0cc9f48f79fdc7cd20836cf6199f19eb4ca4f301"
links: [language-consistency, release-verification]
reviewed_revision: "git:c5d7b90c0b2e126f73fdfd6da850d5eed07b4d61"
status: active
---

# 검증 결과 명확성

Hive 개발 에이전트와 설치된 소비자 하네스: 통과·실패·건너뜀·연기·미검증·미지원
결과마다 영향 범위, 정확한 이유, 현재 실행 환경과의 관계, 실제 실행 여부, 결과가
입증하는 것과 미검증으로 남기는 것을 함께 보고. 운영체제 이름만으로 실행 여부를
대신 표현 금지. 완료 기준: 개발·소비자·프로젝트 지침 일치와 투영 시험 통과.
요청 배경: Windows 검사 결과가 사전 검증인지 운영체제 오인인지 구분되지 않았던
모호한 보고의 재발 방지.
