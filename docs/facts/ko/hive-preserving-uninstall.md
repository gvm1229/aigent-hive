---
schema_version: 1
pair_id: hive-preserving-uninstall
topic_slug: hive-preserving-uninstall
language: ko
counterpart: ../en/hive-preserving-uninstall.md
title: "Hive 보존형 제거"
summary: "제거 확인·소유 감시 종료와 설정·지식·목록·이력 보존"
tags: [bootstrap, onboarding, preservation, uninstall]
aliases: ["clean reinstall", "hive uninstall"]
sources:
  - "repo:crates/hive-cli/src/user_install.rs#sha256:9903f35f13ae28c9853cdf99ab17c42876e8a5b12c91100e7f4c51cc04147c09"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:8393c3c225b6a708473b19181830849e499918f0396a2c226ab7fb4e3afe28d5"
  - "repo:docs/archive/plans/cli-uninstall-0.14.0.md#sha256:db50bb091eff11cd8ee1ee39667b992ac7a947a3ea7b0e7d3d74c020f41956d0"
  - "repo:docs/archive/plans/foundations/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/archive/plans/foundations/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:828eb911da5e50ef6cadd88c301e07f5aa454c2bbe1c9bcd7ef460f7efae00fe"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Hive 보존형 제거

- 0.14.0: 미리보기·확인·소유 감시 종료 후 제거. 지식·선호·프로젝트 목록·이력·외부 바이트 보존. 소유 불명 본체는 미완료. 격리 OS 본체·Windows Codex/감시 제거·복구 수용, 공개 파일은 별도
- 과거 정리 범위·test.19 macOS 빈 agents 말단44개/빈 처리 폴더·동일 표본과0.9.1 Windows 잔존0개 근거는 [UOS-021–022](../../archive/plans/foundations/user-onboarding-shared-index.md)에 보존
- 인증된 빈 부모의 말단 우선 정리, `.hive/dev-install` 개발자 원복 상태 별도 유지
- 0.12.1: 감시 활성 시 제거 거부, 소유 예약 작업 해제 필요
