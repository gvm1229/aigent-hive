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
  - "repo:crates/hive-render/src/lib.rs#sha256:bd58a2ef9a317149e6c76b0d23632f67174e139a9f339770247ffb458e0e9c39"
  - "repo:harness/project-setup/skill-suites.yml#sha256:58ed9002b60fe1b9907c6d9b2cfb1d9846afb5a5b551c98c2c08024968e333c1"
  - "repo:schemas/setup-answers.schema.json#sha256:7f3ec163942955be5ad16c177eafbf648cf209e52c701e6334c651b521e24624"
links: [agent-directive-ownership, codex-skill-delivery]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# 프로젝트 기본 스킬

daily-work의 명시된 기본 스킬 27개. user-setup·project-setup·custom-subagent-create·project-transition 기본 제외. 프로젝트 개별 선택·빈 선택 허용, user-setup 전역 전용. Wiki 비활성 시 지식 3개 제외. 정책 1의 이전 설치 기본 전환 제안과 이후 사용자 선택 보존. 요청에 맞는 스킬 활용과 변경·게시 권한의 구분.
