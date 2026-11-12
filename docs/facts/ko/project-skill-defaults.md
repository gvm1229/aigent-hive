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
  - "repo:crates/hive-render/src/lib.rs#sha256:b6285195847e5b1779bd4a168315a5337eac5885ebacabce45171b22959bf199"
  - "repo:harness/project-setup/skill-suites.yml#sha256:66bb2a743e2b486ec52666b69848b604bcc79bd35d3f741105a88623fc27cf56"
  - "repo:schemas/setup-answers.schema.json#sha256:c4fb1f2da1748a1e087a36d58fb4e63378b2e216e2dfad5bd283e9f711459631"
links: [agent-directive-ownership, codex-skill-delivery]
reviewed_revision: "git:be1a9fc625b4bdb79597be8c3c081381d23e23d3"
status: active
---

# 프로젝트 기본 스킬

daily-work의 명시된 기본 스킬 32개. user-setup·project-setup·custom-subagent-create·knowledge-transfer·project-transition 기본 제외. 프로젝트 개별 선택·빈 선택 허용, user-setup 전역 전용. Wiki 비활성 시 지식 5개 제외. 정책 1의 이전 설치 기본 전환 제안과 이후 사용자 선택 보존. 요청에 맞는 스킬 활용과 변경·게시 권한의 구분.
