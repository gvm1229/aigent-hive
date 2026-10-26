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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:1ab23dbb5a8b60322f6c1068ba7cc09f54bc34151915068a748a1cc5a2c38018"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2fade132e2fe4e60b3f7b70e089d5deaa145a25e2976bdfdb62a58a82fa86192"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:3c6e0b3a0f62d5598e68024fd7a69d82cb2b0dcfbb29d5602f6f362bc0df2d40"
  - "repo:crates/hive-update/src/merge.rs#sha256:d081f88fb284963e9ae02366ae3e99f76f9e65383948660ea59f403df89d741d"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:8c252fa5ef5c4c40647cc11127a404f0bac7c096648e8b3ca9f8af9655203067"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:14b97538abd3ac185627d5dc0a573b540122f780e5549edf80373c6c98866e69"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:f55b00d6b7a34433fe32eb03d0914cc3e8726c879591a46b4c09fb70b4686b82"
links: [consumer-session-coordination, hive-preserving-uninstall]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# 인증된 projection 갱신 정리

전역 폐기 스킬 제거: 폐기 이름 목록·배포된 과거 지문 일치 필수. 프로젝트 갱신: 인증 원본 기준의 미수정 폐기 파일만 삭제, 사용자 수정·외부 파일 보존.

지침·정확한 AGENTS 표시 블록의 겹친 Hive 안전·소유권 규칙은 새 규칙 우선. 분리된 추가·외부 블록·나머지 로컬 충돌은 로컬 우선. 미리보기·지문 승인·원자적 적용·원복 유지.

정리 요청에서 발견한 부속 폴더 오류 수정: 검증된 실제 삭제 파일로 상위 폴더 관계 확인, 연결 추적 금지·빈 폴더만 제거. 실제 프로젝트 검증·두 번째 변경 0건 확인, 모델 준수는 별도 근거 필요.
