---
schema_version: 1
pair_id: hive-preserving-uninstall
topic_slug: hive-preserving-uninstall
language: ko
counterpart: ../en/hive-preserving-uninstall.md
title: "Hive 보존형 제거"
summary: "보존형 제거: Hive-owned transient·retired projection 완전 정리, knowledge·preference·foreign byte 보존."
tags: [bootstrap, onboarding, preservation, uninstall]
aliases: ["clean reinstall", "hive uninstall"]
sources:
  - "repo:crates/hive-cli/src/user_install.rs#sha256:cd9973f27fdd4952ee1936257cb4ed0f2f98d6b8033aa1f89fc2b93eefeb34fd"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:a7caad0d660d83378b297ec3678312fa741b411b8115ed9eca98400d1db703d0"
  - "repo:docs/archive/plans/foundations/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/archive/plans/foundations/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
links: [global-onboarding, knowledge-preservation, release-verification]
reviewed_revision: "git:0ee29cc7a10c6786f5d325569f85182e654cd4b1"
status: active
---

# Hive 보존형 제거

- 제거: Hive 실행 연결·투영·패키지·색인·백업·트랜잭션·실행 상태
- 보존: 지식·저장한 사용자 선호·외부 바이트
- test.19 macOS 점검: 폐기된 빈 agents 말단 44개와 빈 트랜잭션 폴더 잔존
- 설치·갱신·제거에서 인증된 Hive 소유 빈 조상을 말단부터 정리
- 정확한 44개 표본과 0.9.1 Windows 보존 재설치: 폐기된 빈 말단·트랜잭션 폴더 0개
- .hive/dev-install: 별도 개발자 원복 상태
- 0.12.1 감시 활성 시 제거 거부. 소유 예약 작업 해제 뒤 제거, 고아 감시 작업 방지
