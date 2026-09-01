---
schema_version: 1
pair_id: verified-execution-evidence
topic_slug: verified-execution-evidence
language: ko
counterpart: ../en/verified-execution-evidence.md
title: "검증형 실행 적용·종료 근거"
summary: "작업에 연결된 생성·검증 근거로 실행 적용 판정, 개별 재시도 중단과 전체 완료 구분"
tags: [orchestration, skills]
aliases: []
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:4b22be47789033b39654596bb345fd56017e54bf4cd8ef12ad1cac7ae9c8e4d4"
  - "repo:.agents/directives/04-documentation-state.md#sha256:2626e090a19b45a88bc586c0292870dbf6136de40e3aa32359af2f617ead90a3"
  - "repo:harness/skills/verified-workflow/SKILL.md#sha256:fc19bed8a17b8b8652c37ff518528ada2aec511e163b15c99af90235e6728a82"
links: [host-neutral-continuation, verified-workflow]
reviewed_revision: "git:5ea719a64f4403d1261feaff28d3f718d257638a"
status: active
---

# 검증형 실행 적용·종료 근거

지침 교정: 작업에 연결된 생성·검증 영수증이 실제 실행 적용의 선행 근거.
재시도 중단과 전체 종료 구분. 명령 성공 대신 종료 가능 여부와 현재 기준 확인.
소스 작업은 소스 정책 적용, 루트에 소비자 실행 상태 생성 금지.
지원되는 격리 실행 연결이 없으면 검증형 실행 적용 주장 없이 소스 계획에 따라 지속.
지침만으로 호스트의 최종 응답 차단 증명 불가.
