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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:f8920322c1f918b16e9b2df7c1b3a29867cbd4c6cc95b82caa33016d63faab47"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:85b13d22add18756fa11e29fcc1ebcf84b18d143385991143a8453c29e3d0328"
  - "repo:docs/archive/plans/releases/0.9.0/v0.9.0-knowledge-autocapture-regression.md#sha256:44fcfa9e2c19c626eb8a7885afcaeb6405b454748e62349c1459958d4180236c"
  - "repo:harness/skills/knowledge-capture/SKILL.md#sha256:9e86075240574d1e589329ae724c97fac32dab2e2d367b7d878bd84e69d4b483"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:531437bfcb9786cd5221de32eb5ad536bfd07973db159ca0b15a5df858ffa923"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:e5c2c599562121ed3dc43143c16a0b1f063cefa2"
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
