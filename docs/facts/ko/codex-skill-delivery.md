---
schema_version: 1
pair_id: codex-skill-delivery
topic_slug: codex-skill-delivery
language: ko
counterpart: ../en/codex-skill-delivery.md
title: "Codex 스킬 제공과 자동 정리"
summary: "Codex 플러그인 제공과 사용자·프로젝트 갱신의 인증 사본 자동 정리"
tags: [codex, skill, update]
aliases: ["스킬 중복", "자동 정리"]
sources:
  - "repo:crates/hive-render/src/skill_delivery.rs#sha256:39a5ba630384bdd526f3fdec52dc51d09f9b8a9b00e7a66a5c7d2bf6fc1927ff"
  - "repo:docs/decisions/ADR-0024-codex-skill-delivery.md#sha256:a2c66b03a14bbef1564c50fcd2e541c31e1f956d3e29b9da5155a34c0ed42ffd"
links: [global-onboarding, projection-upgrade-purge, public-skill-identity]
reviewed_revision: "git:94fa2196febf288556780c7f68e6f752f15a6415"
status: active
---

# Codex 스킬 제공과 자동 정리

0.11.1 구현의 검증된 활성 Codex 플러그인 기본 스킬 제공. 사용자·프로젝트 각각의 갱신에서 인증된 중복 사본 자동 정리. 수정·원본 불명 파일 보존과 이유 표시. 프로젝트 전용 스킬의 로컬 제공 유지, 플러그인 부재 시 프로젝트 갱신으로 로컬 제공 복원. 등록·파일 시험과 실제 Codex 선택 목록·모델 호출의 별도 검증 필요.
