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
  - "repo:README.md#sha256:a03aae178a8c1060d3f4301d4ed592a24e8cf9e9e95a7b87afa434804ad4ecbb"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:cb42f6c3bd643bc236f3af89f4388ffdbc08db66af88123a38267b904d7b9d01"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:d30564f33f2ead463cfe9e18aa68b697cb07b6c419ee42c9b583fcc11edaf966"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:c28e40cf7e976d0b4455a73c2ddb6d598af448cd549569097784e9a26b2d678e"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:4f3676378fafac75f9c6376210c760a2e0200e843ead0825d1b34d7446864e34"
  - "repo:harness/user-setup/catalog.yml#sha256:4926655a12591cae061e674d774557e96f000d149f8dec1c2b1b650ba235f494"
  - "repo:schemas/user-setup.schema.json#sha256:e83e5f318a5b6ffcc08cfe0898a2b6138512c6bfb0eea99c6070b134f3712f47"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:35f5bce71814a3e874fe53a8730024f16013ad46"
status: active
---

# Global onboarding

순서: CLI 설치, host 활성화, global setup, project setup. Global setup의 project 검사: `0건`.
전역 한도: 사용자 선택, project 한도: 더 이른 중지만 허용. Windows 복구: 질문 전 CLI 확인,
답변별 진행 저장, OS 임시 파일 하나와 정리, product-only Skill, 공통 사용량 보호. Hive marketplace
미완료 활성화: 설정 재개 전 조용히 복구, knowledge·저장 preference·foreign host entry 보존.

`hive uninstall`: Hive-managed setup 상태만 제거, knowledge base·저장 preference 보존. full-purge
flag 없음. 이후 user-scope install: 저장 preference 재사용, setup 질문 생략. Stable `0.9.0`:
test.12 Windows clean reinstall·새 session 수용 증거 필요.
