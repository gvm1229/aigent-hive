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
  - "repo:crates/hive-render/src/skill_delivery.rs#sha256:89274e89fafa3a325edf96917faa943f25bd8531675c5a0ac3fec50fa4b09ecf"
  - "repo:docs/decisions/ADR-0024-codex-skill-delivery.md#sha256:9e4511247d0ed17075ec0b9b6435a612448c18e341fa4fd628c41d91688bc720"
  - "repo:tests/results/legacy/7d159c3524fa28e6fde6.md#sha256:6d07e8da05964857e1e492e6ea596cdc6567947c998ddc39dbcd0bd60ede8bab"
  - "repo:tests/results/legacy/f41c995a82c3db97f0a4.md#sha256:e19a2a3862f6936c502a46ebb443b99d3df992385d79b26e14856a52526e4193"
links: [global-onboarding, projection-upgrade-purge, public-skill-identity]
reviewed_revision: "git:34c4d6cef6c3992402b41c8a801f0b4e57fc68dd"
status: active
---

# Codex 스킬 제공과 자동 정리

Codex 0.11.1의 검증된 활성 플러그인 기본 스킬 제공. 사용자·프로젝트 갱신에서 인증된 동일 원본의 중복만 자동 정리, 수정·불명 파일 보존과 이유 표시. 프로젝트 전용 스킬의 로컬 제공, 플러그인 검증 실패 시 로컬 복원. 호출 정책의 일치도 중복 제거 조건, 불명·차이 시 로컬 유지. 선택 목록의 번역 표현과 호출 정책의 구분.

현재 사용자 test.6 설치의 지식·저장 설정·외부 지침 보존. 새 실제 관리 서버의 Hive 스킬 28개·중복/오류 0개·모델 호출 0건. GUI·명시/자연어 모델 호출은 미검증.
