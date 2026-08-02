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
  - "repo:.agents/directives/01-behavior.md#sha256:d1e3d4cbc89c962bfae66b5a9c135562bd962fa0d8a3765ad2d150e4a9e41195"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:f358b71778a5da093ffdad11470a7f4367573f037f219e4a497dc877bbd86f35"
  - "repo:harness/template/AGENTS.md.jinja#sha256:c53d41177ef323c50041c8e02928fd1db9904188c22d652d5a80dfbd454228e5"
links: [language-consistency, source-development]
reviewed_revision: "git:33f365d3dbb1af51333a6dbb1834ce437a932ea0"
status: active
---

# 사용자 인계 전 자동 처리

개발 에이전트와 설치된 소비자 하네스는 권한 안에서 안전하게 자동화할 수 있는
작업을 모두 끝낸 뒤 남은 일을 제시. 사용자 권한이 필요한 단계만 정확한 위치
또는 명령어, 기대 결과나 확인 근거, 사용자 권한이 필요한 이유와 함께 인계.
실패하거나 수행할 수 없는 항목은 원인과 복구 방법을 분리해 명시. 완료 기준은
개발 지침·소비자 투영 일치와 회귀 시험 통과. 요청 배경은 맥락 없는 미완료
목록 대신 실제로 수행 가능한 안내 제공.
