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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:32986c94309e87a9d4f78c6398c601426490b9172da9e344955a205eafab38d5"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:db91b9841c057a3f9b964185fb2a2f3c2f8701908cf6439e26bf05d389a7243d"
  - "repo:docs/plans/active/v0.9.0-knowledge-autocapture-regression.md#sha256:44fcfa9e2c19c626eb8a7885afcaeb6405b454748e62349c1459958d4180236c"
  - "repo:harness/skills/knowledge-capture/SKILL.md#sha256:3566b5cff1f866a11f2d3ce216759dd7d69cd268b61dc5e428e17703146b836c"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:7b5d334b67e9db1b981273f9fb134adca8129250f241d7359f6f1bc5bda88c1e"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:527434f7364b6be38e7b6941bf48df207c58b32c"
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
