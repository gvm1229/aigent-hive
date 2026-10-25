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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2fade132e2fe4e60b3f7b70e089d5deaa145a25e2976bdfdb62a58a82fa86192"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:9cf945c0072f953b590757cffec4fbda47b68ac0145cb4b573b220aa6ddc4d45"
  - "repo:docs/archive/plans/foundations/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/archive/plans/foundations/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:docs/plans/active/cli-uninstall-0.14.0.md#sha256:1b72c0d2c6c41fad9b764dce1791a044305b9cfa248e5b239591c29969df7b82"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:b33ed5b0aa3819b3607100012f444faa6af82f7f1363c2ca492cb7eaf81bb1df"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Hive 보존형 제거

- 0.14.0: 미리보기·확인·소유 감시 종료 후 제거. 지식·선호·프로젝트 목록·이력·외부 바이트 보존. 소유 불명 본체는 미완료, 실제 OS 제거 수용 대기
- 과거 정리 범위·test.19 macOS 빈 agents 말단44개/빈 처리 폴더·동일 표본과0.9.1 Windows 잔존0개 근거는 [UOS-021–022](../../archive/plans/foundations/user-onboarding-shared-index.md)에 보존
- 인증된 빈 부모의 말단 우선 정리, `.hive/dev-install` 개발자 원복 상태 별도 유지
- 0.12.1: 감시 활성 시 제거 거부, 소유 예약 작업 해제 필요
