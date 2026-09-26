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
  - "repo:.agents/directives/01-behavior.md#sha256:49c79d8137a1e1d3cdcb06b60fb7e2708e2cf0ae04018a6c9c866ecb0a65a333"
  - "repo:.agents/directives/references/run-closure.md#sha256:e55fe262c9a36e7a1c0f372941cc7334f5c4f78d888872add82635c0015865a2"
  - "repo:harness/skills/verified-workflow/SKILL.md#sha256:b540e5ca68afee2e3947932e9b21bef1c5707cbde322d7c89cd287965609d5cf"
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
