---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: ko
counterpart: ../en/global-onboarding.md
title: "Global onboarding"
summary: "전역 설정: 질문 전 signed CLI 확인, 답변별 진행 상태 보존, 사용자 선택 사용량 보호 한도 관리. 프로젝트는 더 이른 중지만 선택 가능."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:67c09e54e76df72ee9ac6acbde5b88fbb0a6653e1d7172e3f789a8d99c2434b7"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:48a76cd5503858a2327c7562879de259334b182687aace98ec1df06b71dd1600"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:d30564f33f2ead463cfe9e18aa68b697cb07b6c419ee42c9b583fcc11edaf966"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:59812ce78d64825be25dbb6576013869e4d334b82868f663281c42c0b1df4e16"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:246f1fa6c352c29a905d4e3981312a2288e701785fb9d95c450d4023a37a059b"
  - "repo:harness/user-setup/catalog.yml#sha256:7dc82dbf559075ce4286e7dd19aec0ddc22e04f35ad4a8a60f43129a4dba2a1f"
  - "repo:schemas/user-setup.schema.json#sha256:46b360a9f91e154d1440e2997b56a964edd122383ccfc9b105b4e2ae4f8939f9"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:35f5bce71814a3e874fe53a8730024f16013ad46"
status: active
---

# Global onboarding

- 순서: CLI 설치, host 활성화, global setup, project setup
- Global setup의 project 검사: `0건`
- 전역 한도: 사용자 선택. Project 한도: 더 이른 중지만 허용
- Mac build 원본 복구: `BGR-008–013` 완료, 현재 validation 통과
- Mac 후속 범위: 신규 구현이 아닌 회귀 방지 gate

Windows test.5 문제: CLI 탐색 실패, schema 추측, 임시 파일, 진행 손실. 수정 조건: 질문 전 CLI
확인, 답변별 진행 저장, OS 임시 파일 하나와 정리, product-only Skill, 공통 사용량 보호.
Stable 0.9.0 조건: maintainer의 Windows 11 fresh test 통과. 이 Mac: source 회귀만 실행.
인증 불가·사용자 수정 byte는 보존.
