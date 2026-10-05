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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:07c53684b15ca76c0912c67ab9d2dac2464a98580fa186956492a95fa238af63"
  - "repo:docs/archive/plans/releases/0.9.0/v0.9.0-knowledge-autocapture-regression.md#sha256:44fcfa9e2c19c626eb8a7885afcaeb6405b454748e62349c1459958d4180236c"
  - "repo:harness/skills/knowledge-capture/SKILL.md#sha256:40d08d401f60962d70f8bb4fcb83b4f17b3b067d165e24db9f579cf364b63b2d"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:b1c993c5c9332596f2f71f0db2e40f42a434e210f71f4e8bb9f38da763c4b673"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:d147dfc9d41b61990251c0c0ef5161c8f0d4f3da"
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
