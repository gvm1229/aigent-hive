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
links: [global-onboarding, projection-upgrade-purge, public-skill-identity]
reviewed_revision: "git:94fa2196febf288556780c7f68e6f752f15a6415"
status: active
---

# Codex 스킬 제공과 자동 정리

0.11.1 구현의 검증된 활성 Codex 플러그인 기본 스킬 제공. 사용자·프로젝트 각각의 갱신에서 인증된 중복 사본 자동 정리. 수정·원본 불명 파일 보존과 이유 표시. 프로젝트 전용 스킬의 로컬 제공 유지, 플러그인 부재 시 프로젝트 갱신으로 로컬 제공 복원. 등록·파일 시험과 실제 Codex 선택 목록·모델 호출의 별도 검증 필요.

중복 제거의 필수 호출 정책 일치 조건. 미일치·누락 시 로컬 제공·사유 기록, 선택 화면의 번역 문구와 호출 정책 구분.
