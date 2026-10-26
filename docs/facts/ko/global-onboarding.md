---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: ko
counterpart: ../en/global-onboarding.md
title: "전역 설정"
summary: "신규0.14.0 자동 중단 비활성과 기존 선택 재검토, 사용자 자료 보존"
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:049596fcbb0b062c936adef3eccc3fb154e407778a1c98bbfef6da20314fd07f"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:3c6e0b3a0f62d5598e68024fd7a69d82cb2b0dcfbb29d5602f6f362bc0df2d40"
  - "repo:crates/hive-core/src/preferences.rs#sha256:838005e60b3255422c2ee143d76e66ef3ef877d7f3a8ff196eb446c0dfdc888d"
  - "repo:docs/archive/plans/foundations/native-usage-sensor.md#sha256:231e96967c32029d539eb82f245399e37156a43c2028be8a01a51215a5455807"
  - "repo:docs/archive/plans/foundations/usage-guard-policy.md#sha256:4b99d1f046ff56eeb9102b99dec4e88226ca2cdfa4947bb233c9a5c541a19172"
  - "repo:docs/archive/plans/foundations/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/archive/plans/foundations/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
  - "repo:docs/decisions/product-release-decisions.md#sha256:3620f38dc575abdd65508175d3a33b7e998b957d70b05c8c807337eafa5a0321"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:14b97538abd3ac185627d5dc0a573b540122f780e5549edf80373c6c98866e69"
  - "repo:harness/user-setup/catalog.yml#sha256:fed2aefc7efa52c28bb05c5b069ad4c4fbeec30b805fff7b84d00285fca18ea4"
  - "repo:schemas/user-setup.schema.json#sha256:4f9c5f59802bc964cfdc41636c32e7e71929c52d64a0291babba948906567a1f"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# 전역 설정

- 0.14.0 이전: 빠른 설정의 보호 활성화·남은 사용량20% 권장, 맞춤 설정의 한도 질문
- 신규0.14.0: 자동 중단 비활성. 기존 선택·출처 불명의 CLI 재검토, 비대화형 갱신의 선택 보존
- 기본 조회 실패 확인과 별도 동의 전 CodexBar 비노출
- 설정·갱신·제거의 현재 Skill 구성 유지, 인증된 폐기 빈 폴더 정리, 지식·선호·외부 바이트 보존

- 이미 받은 선택은 CLI의 `answer_patch_template`에서 형식·판 번호를 유지하고 answers에만 입력, 파일 경로를 `--answers`로 전달. 사용자 값을 추측하거나 표제에서 키를 만들어 넣지 않는 계약
