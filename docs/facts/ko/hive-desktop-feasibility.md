---
schema_version: 1
pair_id: hive-desktop-feasibility
topic_slug: hive-desktop-feasibility
language: ko
counterpart: ../en/hive-desktop-feasibility.md
title: "데스크톱 실행 앱 전환 조사"
summary: "외부 실행 엔진 연결형 앱과 Hive 자체 에이전트 실행 엔진의 구분"
tags: [architecture, desktop, research, runtime]
aliases: [Hive Desktop]
sources:
  - "repo:docs/research/hive-desktop/README.md#sha256:a17b9c9a2a5ebace89510af320604ee0c1644f90e58793f712c1484e7d89b100"
  - "repo:docs/research/hive-desktop/changes.md#sha256:9eac567a47a10efb98e7897a8004223b0ec37a43d888354bf557fb9595d779e5"
  - "repo:docs/research/hive-desktop/runtime.md#sha256:46b736a53674c87f21e847f13eb41c0964619c0fb4964ead224505113776b323"
links: [orchestration-ownership, product-non-goals]
reviewed_revision: "git:88f47770af645627980e3627b02d334dd2ce9001"
status: active
---

# 데스크톱 실행 앱 전환 조사

- 사용자 요청: 완전한 데스크톱 앱의 가능성 조사
- [보고서](../../research/hive-desktop/README.md): 외부 엔진 연결형과 Hive 자체 실행 엔진의 구분
- 기술 후보: Tauri 2·React·TypeScript·공통 Rust 업무 처리
- 근거: Windows 코드 읽기·공식 문서
- 증명 제외: 앱 빌드·모델 실행·인증 자격·성능·일정·macOS와 Linux 수용
- 실행 정책 변경·새 제품 버전의 미승인 상태
