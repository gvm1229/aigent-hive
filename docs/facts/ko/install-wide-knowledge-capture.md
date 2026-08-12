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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:a00d240fa71fecf28877a43253cdc20190279d9e3d5d0b63bf0ad8a47ab9b7de"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:4aecfd684f8c07326a639e92061de5f2ea52050cddc352a3b2f4b6b4adb1d3c2"
  - "repo:harness/skills/knowledge-capture/SKILL.md#sha256:d2e23636ac998bf0b8cca29cf2466e761ab6abe6f20313ad0c9b4d2c6cf71459"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:f06146778f6faf907e402462008e970bc82cf134f9e8cb9c31a3b727b20e66ec"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:527434f7364b6be38e7b6941bf48df207c58b32c"
status: active
---

# 설치 범위 지식 수집

전역 위키 활성 상태의 선택 호스트 사용자 지침: 설치 직후 모든 폴더의 매 턴 검토. 프로젝트 설정,
Hive harness, project marker, 연결 collection: 안전한 user-root 수집의 전제 조건 아님. 미등록 target의
자동 조회: project-private·confidential 지식을 제외한 user-root·shared collection 검색. 수집 방식:
foreground agent 검토, 정규화, 제한된 범위. Background raw-prompt recorder: 미사용.
