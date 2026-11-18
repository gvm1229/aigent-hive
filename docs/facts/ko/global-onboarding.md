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
  - "repo:README.md#sha256:ad3e635304ff898ed338c383f008a18e423e7ed74cb806e9c55fd1543e4b5d73"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:acbf2bce77a81d61db65f3f32150fbded0cfb11ae5df426dd2ca3f7b9a25faee"
  - "repo:crates/hive-core/src/preferences.rs#sha256:cb912cc72fc576c28ddc2436597326957cdd58bca1810d5408e5127692825016"
  - "repo:docs/archive/plans/foundations/native-usage-sensor.md#sha256:231e96967c32029d539eb82f245399e37156a43c2028be8a01a51215a5455807"
  - "repo:docs/archive/plans/foundations/usage-guard-policy.md#sha256:4b99d1f046ff56eeb9102b99dec4e88226ca2cdfa4947bb233c9a5c541a19172"
  - "repo:docs/archive/plans/foundations/user-onboarding-shared-index.md#sha256:2253508f42511c793d5e96739eb3316d149e8112736926e6c04199232cf7326a"
  - "repo:docs/archive/plans/foundations/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
  - "repo:docs/decisions/product-release-decisions.md#sha256:3620f38dc575abdd65508175d3a33b7e998b957d70b05c8c807337eafa5a0321"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:005f05fa4631834f57fd4e8524e51a839d030e88ad1cf193c831a176fce03848"
  - "repo:harness/user-setup/catalog.yml#sha256:4c3a188bcdb06752531530478c34cedafcaafb259fff00ff8743ed2d2e4c85c0"
  - "repo:schemas/user-setup.schema.json#sha256:02903b420365725d94b3cb79000661970338767fe382f163a36c9d58e5288aa8"
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
