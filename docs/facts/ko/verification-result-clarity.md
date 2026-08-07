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
  - "repo:.agents/directives/01-behavior.md#sha256:aff2586323a4db2acad51cd0225b9791e4d0a974cd1a2e96d92eeaafbacdf5d6"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:282c641531d7a72009ee3973801ee8d20ea1305fd7e6f836f202e6a4e22508d4"
  - "repo:docs/guidance-schema.md#sha256:fd8fffda818038ee48b66b0581787e2fd741404b0a9253ca34c0d55f15ad4d15"
  - "repo:harness/template/AGENTS.md.jinja#sha256:64f33fed294900badc58d8ff6b4f7144d0c43bf003884abdcae5c703a60cdd7a"
links: [language-consistency, release-verification]
reviewed_revision: "git:35e6b79a024350487f823780101a28be24a9f4c7"
status: active
---

# 검증 결과 명확성

Hive 개발 에이전트와 설치된 소비자 하네스: 통과·실패·건너뜀·연기·미검증·미지원
결과마다 영향 범위, 정확한 이유, 현재 실행 환경과의 관계, 실제 실행 여부, 결과가
입증하는 것과 미검증으로 남기는 것을 함께 보고. 운영체제 이름만으로 실행 여부를
대신 표현 금지. 완료 기준: 개발·소비자·프로젝트 지침 일치와 투영 시험 통과.
요청 배경: Windows 검사 결과가 사전 검증인지 운영체제 오인인지 구분되지 않았던
모호한 보고의 재발 방지.
