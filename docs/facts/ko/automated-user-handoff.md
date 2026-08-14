---
schema_version: 1
pair_id: automated-user-handoff
topic_slug: automated-user-handoff
language: ko
counterpart: ../en/automated-user-handoff.md
title: "사용자 인계 전 자동 처리"
summary: "안전하게 자동화할 수 있는 작업을 먼저 끝내고 사용자 권한이 필요한 단계만 간결하게 인계."
tags: [automation, behavior, handoff]
aliases: ["사용자 수행 단계", "할 일 인계"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:42bbd59e702cdce48ac6396d4c5a2f3a9b7574cd99272e22f3279c00b041cba4"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b8bb2ace9f509cf8f48cf703971069e7ca73ada3704a8c7dc18adfd03a27e9ef"
  - "repo:harness/template/AGENTS.md.jinja#sha256:33c0da7ba5156ea1aa0ccc08a8e4f88343cf5f6f896994a7d8b830ac0ad6bb74"
links: [language-consistency, source-development]
reviewed_revision: "git:35e6b79a024350487f823780101a28be24a9f4c7"
status: active
---

# 사용자 인계 전 자동 처리

개발 에이전트와 설치된 소비자 하네스는 권한 안에서 안전하게 자동화할 수 있는
작업을 모두 끝낸 뒤 남은 일을 제시. 사용자 권한이 필요한 단계만 정확한 위치
또는 명령어, 기대 결과나 확인 근거, 사용자 권한이 필요한 이유와 함께 인계.
실패하거나 수행할 수 없는 항목은 원인과 복구 방법을 분리해 명시. 완료 기준은
개발 지침·소비자 투영 일치와 회귀 시험 통과. 요청 배경은 맥락 없는 미완료
목록 대신 실제로 수행 가능한 안내 제공.
