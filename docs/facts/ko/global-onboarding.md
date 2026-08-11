---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: ko
counterpart: ../en/global-onboarding.md
title: "Global onboarding"
summary: "전역 설정: 질문 전 signed CLI 확인, 답변별 진행 상태 보존, 보존형 Hive 제거 뒤 저장 preference 재사용."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:e42b36f0bfd2629b0abbbeb2010d8aae5bac3086982f1a714bb6888aef1a3364"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:0bfd9117a0d835da5f19bc02b82959a5630a4955a81ee3efda0a6ba5246dfaad"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:6b2a26d1285073e6796f683abfc190bd6d74a05d57b83900412da37aa5d53849"
  - "repo:harness/user-setup/catalog.yml#sha256:4926655a12591cae061e674d774557e96f000d149f8dec1c2b1b650ba235f494"
  - "repo:schemas/user-setup.schema.json#sha256:e83e5f318a5b6ffcc08cfe0898a2b6138512c6bfb0eea99c6070b134f3712f47"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:01df1d580d987e7fb0f34978076cd000263fd99f"
status: active
---

# Global onboarding

순서: CLI 설치·host 활성화·global setup·project setup. Global setup의 project 검사: `0건`.
전역 한도: 사용자 선택, project 한도: 더 이른 중지만 허용. Windows 복구: 질문 전 CLI 확인,
답변별 진행 저장·OS 임시 파일 하나·product-only Skill·공통 사용량 보호·미완료 marketplace 조용한 복구,
knowledge·저장 preference 보존.

`hive uninstall`: Hive-managed setup 상태만 제거, knowledge base·저장 preference 보존. full-purge
flag 없음. 이후 user-scope install: 저장 preference 재사용, setup 질문 생략. Windows test.12 수용:
새 session 탐색·Discord 실제 전달 포함. 다음 출시 수용: contributor 입력 없는 product-owned 신속 기본 profile.
