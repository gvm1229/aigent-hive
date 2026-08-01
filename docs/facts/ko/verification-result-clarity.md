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
  - "repo:.agents/directives/01-behavior.md#sha256:a78fc02202dc5c3b934e28924dd86660d297151f4905606dc7a26f2179083eaa"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:e21faccc9dae23d7522de433e345890509ce8d742fa8fe6a375f0892e35713db"
  - "repo:docs/guidance-schema.md#sha256:aca1d198c2fc72a5bde7f63d128467ad297454ec5ce5c7a55c6b010a022f0f2a"
  - "repo:harness/template/AGENTS.md.jinja#sha256:9e5694a62099d262872bd6e1f167d839d9eb3f51c3d6cdfd4884656350cc0ec4"
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
