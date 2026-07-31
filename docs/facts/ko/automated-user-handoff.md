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
  - "repo:.agents/directives/01-behavior.md#sha256:d59f86031a7bb6f889eeaa00598794fdd2f73375da7d03cdb6a5b49d4884dc0f"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:0ed886384328d10f394f0f2f8fb6f1deed69908af026ecaa17e1e75e17b39a3a"
  - "repo:harness/template/AGENTS.md.jinja#sha256:6198d9b0380ee4e46d44a6aab9ea759c0080690e3353a9309da1a12c5b1939c2"
links: [language-consistency, source-development]
reviewed_revision: "git:8c190672e3f08ade9bdf985016bcf7b00fa157a1"
status: active
---

# 사용자 인계 전 자동 처리

개발 에이전트와 설치된 소비자 하네스는 권한 안에서 안전하게 자동화할 수 있는
작업을 모두 끝낸 뒤 남은 일을 제시. 사용자 권한이 필요한 단계만 정확한 위치
또는 명령어, 기대 결과나 확인 근거, 사용자 권한이 필요한 이유와 함께 인계.
실패하거나 수행할 수 없는 항목은 원인과 복구 방법을 분리해 명시. 완료 기준은
개발 지침·소비자 투영 일치와 회귀 시험 통과. 요청 배경은 맥락 없는 미완료
목록 대신 실제로 수행 가능한 안내 제공.
