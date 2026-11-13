---
schema_version: 1
pair_id: project-skill-defaults
topic_slug: project-skill-defaults
language: ko
counterpart: ../en/project-skill-defaults.md
title: "프로젝트 기본 스킬"
summary: "프로젝트 기본 스킬"
tags: [directive, project, skill]
aliases: []
sources:
  - "repo:crates/hive-render/src/lib.rs#sha256:806b1344ca824075a623199e994caa725adce1335bdda6fe4f5b5fb7504ff366"
  - "repo:harness/project-setup/skill-suites.yml#sha256:7ee263e954cf1dbba42022fdbdd19f53cafd9064b2345c7c5e0cd0db36186f84"
  - "repo:schemas/setup-answers.schema.json#sha256:99da8a488720b5c0f7891a73108caacfcd8f368fc415ebbf11d20cd09783f737"
links: [agent-directive-ownership, codex-skill-delivery]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# 프로젝트 기본 스킬

daily-work의 명시된 기본 스킬 32개. user-setup·project-setup·custom-subagent-create·knowledge-transfer·project-transition 기본 제외. 프로젝트 개별 선택·빈 선택 허용, user-setup 전역 전용. Wiki 비활성 시 지식 5개 제외. 정책 1의 이전 설치 기본 전환 제안과 이후 사용자 선택 보존. 요청에 맞는 스킬 활용과 변경·게시 권한의 구분.
