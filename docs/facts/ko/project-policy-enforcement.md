---
schema_version: 1
pair_id: project-policy-enforcement
topic_slug: project-policy-enforcement
language: ko
counterpart: ../en/project-policy-enforcement.md
title: "프로젝트 전체 정책 검사 설계"
summary: "호스트 훅의 규칙 전달과 도메인 검증·최종 변경 지점의 강제 검사를 분리하는 검토 설계"
tags: [architecture, hooks, policy]
aliases: []
sources:
  - "repo:docs/research/project-policy-enforcement-2026-09-18.md#sha256:0abd7c3992bfa2ac3859a472bb48d624651bba994e817a83e7fe51f93b481311"
links: [artifact-boundaries, foundation-refactor]
reviewed_revision: "git:23a94c06874fd65f8cdb0151f8512caed88fd792"
status: active
---

# 프로젝트 전체 정책 검사 설계

- `0.11.0` 검토 설계: 소스·소비자·사용자·출시별 규칙 적용 범위 구분
- 훅은 규칙 전달·이벤트 변환, 기존 도메인 검사와 최종 변경 지점은 지원 작업의 실제 강제 담당
- 지침 로드·호출자의 성공 주장만으로 준수 증명 불가, 의미 품질은 별도 검토
- 이번 결과는 설계 제안, 제품 전체 훅 구현·실제 호스트 수용과 구분
