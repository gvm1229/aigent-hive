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
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:2150681617bd1c2273780f0796609f27fc4815418428c0743ef11b88245deb38"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:19537ff592e3146740b87256c5cd25033ceb8dbfc556c7da7f219baf1360666e"
  - "repo:harness/skills/configure/SKILL.md#sha256:abeb032e21d2576366025465d54080966767fb7e17cca57848acf093eaa83eaf"
  - "repo:harness/user-setup/catalog.yml#sha256:7dc82dbf559075ce4286e7dd19aec0ddc22e04f35ad4a8a60f43129a4dba2a1f"
  - "repo:schemas/user-setup.schema.json#sha256:46b360a9f91e154d1440e2997b56a964edd122383ccfc9b105b4e2ae4f8939f9"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
status: active
---

# Global onboarding

- 순서: CLI 설치, host 활성화, global setup, project setup
- Global setup의 project 검사: `0건`
- 전역 한도: 사용자 선택. Project 한도: 더 이른 중지만 허용

Windows 11 test.5 문제: CLI 탐색 실패, schema 추측, 임시 답안 파일, 진행 손실. 수정 조건: 질문 전
CLI·signed setup metadata 확인, 답변별 진행 저장, OS 임시 파일 하나와 cleanup, product-only Skill
목록, 공통 전역·project guard. Stable 0.9.0 조건: maintainer의 실제 Windows 11 machine에서 fresh
numbered-test 통과. 이 Mac: source·cross-platform 회귀만 실행. 인증 불가·사용자 수정 byte는 보존.
