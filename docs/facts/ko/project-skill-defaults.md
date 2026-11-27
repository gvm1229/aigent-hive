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
  - "repo:crates/hive-render/src/lib.rs#sha256:95986fc182da052aa80fb7d2e57dae3d1b65cb3a8f9226932ca2914e32050132"
  - "repo:harness/project-setup/skill-suites.yml#sha256:8b044a217f41e45c39ed8f9ae0f465d38a592d7b9484e4d97d918234aeab381a"
  - "repo:schemas/setup-answers.schema.json#sha256:1a14afc5c389eaad085a2daedfa0c8e14700fdbaabd8f81a779634c793d51895"
links: [agent-directive-ownership, codex-skill-delivery]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# 프로젝트 기본 스킬

daily-work의 명시된 기본 스킬 27개. user-setup·project-setup·custom-subagent-create·project-transition 기본 제외. 프로젝트 개별 선택·빈 선택 허용, user-setup 전역 전용. Wiki 비활성 시 지식 3개 제외. 정책 1의 이전 설치 기본 전환 제안과 이후 사용자 선택 보존. 요청에 맞는 스킬 활용과 변경·게시 권한의 구분.
