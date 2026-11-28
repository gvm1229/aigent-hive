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
  - "repo:.agents/directives/01-behavior.md#sha256:54f993c4403e7de3612efa12f499bbf1c3a957ee5893875cb0c50204505b1f40"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:4fd5dd55857ac73d960d702e697b1cd05502f68806844c17c1bd199f8d1f4021"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:9c5854224d063f07088155f4a61c672587e8bf26c2a67b1c4fb88be9e6a4d4e4"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:d263c84680b1b11fc49175ee181955a2b18fd90e819b1929212fd818b176e3fd"
  - "repo:harness/template/AGENTS.md.jinja#sha256:fe7f06cc88b566a9ab1ef2919d8ca43830aa963a98208c084c56b5c12f26c864"
links: [language-consistency, source-development]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# 사용자 인계 전 자동 처리

개발 에이전트와 설치된 소비자 하네스는 권한 안에서 안전하게 자동화할 수 있는
작업을 모두 끝낸 뒤 남은 일을 제시. 사용자 권한이 필요한 단계만 정확한 위치
또는 명령어, 기대 결과나 확인 근거, 사용자 권한이 필요한 이유와 함께 인계.
실패하거나 수행할 수 없는 항목은 원인과 복구 방법을 분리해 명시. 완료 기준은
개발 지침·소비자 투영 일치와 회귀 시험 통과. 요청 배경은 맥락 없는 미완료
목록 대신 실제로 수행 가능한 안내 제공.

사용량 소진 시 인계 기록·정리도 연기. 추가 도구 없이 기존 보호 기록 보존, 짧은 중단 안내와 작업 완료 주장의 구분.
