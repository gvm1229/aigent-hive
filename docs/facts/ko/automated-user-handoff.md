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
  - "repo:.agents/directives/01-behavior.md#sha256:e92fc32054100c81742fa37aac4354bd971feafe62d887b7b6c8f6aa65882e49"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:037c9a991a929a76aee9da633a6ecb666e3b45d02315c0e721934eccedd2245a"
  - "repo:harness/template/AGENTS.md.jinja#sha256:bb858b1021be8b3fd9fc282820a34a4e923dea6a47e01bdddcf9745510c1381d"
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
