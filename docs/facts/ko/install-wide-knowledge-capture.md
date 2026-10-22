---
schema_version: 1
pair_id: install-wide-knowledge-capture
topic_slug: install-wide-knowledge-capture
language: ko
counterpart: ../en/install-wide-knowledge-capture.md
title: "설치 범위 지식 수집"
summary: "Hive 사용자 범위 수집·조회는 프로젝트 설정 없이 설치 직후 선택 호스트의 모든 프로젝트에 적용."
tags: [capture, knowledge, retrieval, user-root]
aliases: ["미등록 프로젝트 조회", "설정 독립 지식"]
sources:
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:dff827b04011d7ad8ff7412543aaccfe570d63a10bb0370f4c52409544504cb4"
  - "repo:crates/hive-cli/src/knowledge/remember.rs#sha256:b3f259173a92bf050ca26df5b462330ea4ce9d96bb9bca431b6a54700ddfea36"
  - "repo:crates/hive-cli/src/knowledge/retrieve.rs#sha256:a72952ecc2423f82bd6710e9d88a88cd6b59f0d87500b30c1a1d68151879bcf1"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:2bfa879fe9bc0d682e6562c3d323d2400daf80478140a3a1d1fc5aa7b09621ed"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:cd9973f27fdd4952ee1936257cb4ed0f2f98d6b8033aa1f89fc2b93eefeb34fd"
  - "repo:docs/archive/plans/releases/0.9.0/v0.9.0-knowledge-autocapture-regression.md#sha256:44fcfa9e2c19c626eb8a7885afcaeb6405b454748e62349c1459958d4180236c"
  - "repo:harness/skills/knowledge-capture/SKILL.md#sha256:7231f60877fd7d269ff3d3b9cc7851daefb42f75baef1682dacb4d9644feffe6"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:b1c993c5c9332596f2f71f0db2e40f42a434e210f71f4e8bb9f38da763c4b673"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:332849449782b95d9d5819316151e0e8991477fa2cc56ba5df568dcc35f4c237"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:f6bec784d94c76ef79cda8ff28e60a0b05765805"
status: active
---

# 설치 범위 지식 수집

전역 위키 활성 상태의 선택 호스트 사용자 지침: 설치 직후 모든 폴더의 매 턴 검토. 프로젝트 설정,
Hive harness, project marker, 연결 collection: 안전한 user-root 수집의 전제 조건 아님. 미등록 target의
자동 조회: project-private·confidential 지식을 제외한 user-root·shared collection 검색. 수집 방식:
foreground agent 검토, 정규화, 제한된 범위. Background raw-prompt recorder: 미사용.

`0.9.1` Windows 수용: knowledge 명령·Skill 요청이 아닌 PortareFolium 경력 배경 일반 발화 사용.
동일 session의 user-root claim·index receipt 자동 생성과 별도 fresh Codex session의 미등록
project 자동 회수 PASS.

할당량 소진 확인 시 지식 기록과 검사도 연기. 차단된 응답에서 추가 도구 호출 금지, 새 요청과 정상 사용량 확인 뒤 재개.
