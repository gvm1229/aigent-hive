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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:7861ff887f1d831bde68add39190f2678969e08d9fbf4a25c0f74cea04c13077"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2ac47f0ba3f6a05f76c1e524ad9945d695e150c5665ed77dfb496e86ebab82d9"
  - "repo:harness/skills/knowledge-capture/SKILL.md#sha256:7af3911e970abf824597259c041a7c1dda5216cc335a5057e0a7626b9062c2f9"
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
