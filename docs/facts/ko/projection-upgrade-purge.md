---
schema_version: 1
pair_id: projection-upgrade-purge
topic_slug: projection-upgrade-purge
language: ko
counterpart: ../en/projection-upgrade-purge.md
title: "인증된 projection 갱신 정리"
summary: "Hive는 이전 Hive projection을 인증한 뒤 retired Skill과 직접 충돌하는 안전·소유권 규칙만 갱신"
tags: [consumer-harness, preservation, skills, upgrade]
aliases: ["PUG93"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:945222ab2da1310babab674c81b28304d03015271720795afde44abd2324e0e1"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:193eb021ec2fc367176127ae05ab9dc2baea6097819ddd63de43d9a79ee7cc9a"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:b7a409e044561428f1bda83e8daef5a647a63c23a8f1d1426baffb336b63323a"
  - "repo:crates/hive-update/src/merge.rs#sha256:f9c983dc6fb71aa50994332395faaa55058d8e95add818221bed9c8073510558"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:8c252fa5ef5c4c40647cc11127a404f0bac7c096648e8b3ca9f8af9655203067"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:828eb911da5e50ef6cadd88c301e07f5aa454c2bbe1c9bcd7ef460f7efae00fe"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:746a446b6d86020d9536cdbea8bff82d822075338d8f4bbfdaa3202f5dafffd0"
links: [consumer-session-coordination, hive-preserving-uninstall]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# 인증된 projection 갱신 정리

전역 폐기 스킬 제거: 폐기 이름 목록·배포된 과거 지문 일치 필수. 프로젝트 갱신: 인증 원본 기준의 미수정 폐기 파일만 삭제, 사용자 수정·외부 파일 보존.

지침·정확한 AGENTS 표시 블록의 겹친 Hive 안전·소유권 규칙은 새 규칙 우선. 분리된 추가·외부 블록·나머지 로컬 충돌은 로컬 우선. 미리보기·지문 승인·원자적 적용·원복 유지.

정리 요청에서 발견한 부속 폴더 오류 수정: 검증된 실제 삭제 파일로 상위 폴더 관계 확인, 연결 추적 금지·빈 폴더만 제거. 실제 프로젝트 검증·두 번째 변경 0건 확인, 모델 준수는 별도 근거 필요.
