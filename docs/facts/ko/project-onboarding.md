---
schema_version: 1
pair_id: project-onboarding
topic_slug: project-onboarding
language: ko
counterpart: ../en/project-onboarding.md
title: "프로젝트 설정"
summary: "refresh의 감지 사실 갱신과 기록된 종류 선택 보존."
tags: [onboarding, project]
aliases: ["프로젝트 설정"]
sources:
  - "repo:crates/hive-render/src/lib.rs#sha256:29da4ca9b29f58ec853dab54eef3cbfa0119896206ab352335bb1cb05a50b817"
  - "repo:docs/decisions/ADR-0009-user-plugin-project-knowledge-boundary.md#sha256:68d3ff4ada0f4f64af24a1f11b881c51ae650de438af02c7bf0b84a459c5cc0f"
  - "repo:docs/research/overhaul/OV151-007.md#sha256:69efaab07a728c908284c141ffa519f5fcf2b313a8cb0ec23d486e4bc489ff55"
links: [global-onboarding, shared-index]
reviewed_revision: "git:43a894eb75b9fb5bd6327d47d97ce3665fa7a6e3"
status: active
---

# 프로젝트 설정

전역 선택 상속과 미해결 필수 질문 뒤 쓰기 미리보기. 0.15.1 소스 refresh의 명시·수락·현재 버전 기본 종류 결정 보존. 결정 부재 또는 0.15.1 이전 미확인 기본값만 general에서 coding으로 이전, 감지 사실 갱신과 제안 표시. Windows 소스 회귀 시험의 선택 보존 증명, 세 OS 공개 수용·실제 호스트 Skill 발견 미검증. 이전 30종 구현 측정의 연결된 단계 근거 보존. 요청 배경: 단계 4 검토 수정.
