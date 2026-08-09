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
  - "repo:README.md#sha256:a03aae178a8c1060d3f4301d4ed592a24e8cf9e9e95a7b87afa434804ad4ecbb"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:3e05d7beb0322270572036bf73dac4854b5468d1092bb2d940baad4492ca0e55"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:d30564f33f2ead463cfe9e18aa68b697cb07b6c419ee42c9b583fcc11edaf966"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:bc5180991fddb1c2e4132fb0e6f23d4d4d06bd9d60311085d853481201e5c052"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:9ffa22cf14504ba7385135c1f62fdcb19bede32a0925ed72eb23fa8b96359eb5"
  - "repo:harness/user-setup/catalog.yml#sha256:4926655a12591cae061e674d774557e96f000d149f8dec1c2b1b650ba235f494"
  - "repo:schemas/user-setup.schema.json#sha256:e83e5f318a5b6ffcc08cfe0898a2b6138512c6bfb0eea99c6070b134f3712f47"
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
