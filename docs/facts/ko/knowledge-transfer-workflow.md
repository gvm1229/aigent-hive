---
schema_version: 1
pair_id: knowledge-transfer-workflow
topic_slug: knowledge-transfer-workflow
language: ko
counterpart: ../en/knowledge-transfer-workflow.md
title: "컴퓨터 간 지식 이전 흐름"
summary: "단일·여러 묶음의 입력·대상·검토 지문과 벡터 지연 선택을 보존하는 지식 이전"
tags: [knowledge, portability]
aliases: []
sources:
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:f0e47ded9439c9d2fcb2c1be6eb93d11609e942d5320f452fd45feecc7bf7d8a"
  - "repo:crates/hive-cli/src/knowledge_transfer.rs#sha256:d1a6df6babfbed54b46bb505889921a30fe86fd14fbd4cc0230d51bf7a99de92"
  - "repo:crates/hive-wiki/src/bundle_store.rs#sha256:ef8382c6270681076f45da459af68ad0d058b5236a239b7c76b53de056daba1e"
  - "repo:docs/guides/knowledge-transfer.md#sha256:18fcddede882c3dbcfa642b5b6c2b6be6e4bac898532e03c3f178da56c8633af"
  - "repo:harness/skills/knowledge-transfer/SKILL.md#sha256:7b4bbe52c0e4af139f61ded9ba5c75562d21c8e0011530af6512563bbaea7188"
links: [global-knowledge-bundle-transfer, knowledge-storage]
reviewed_revision: "git:523892f0009d7ee04af9381981cb41ba01c4045d"
status: active
---

# 컴퓨터 간 지식 이전 흐름

기존 Markdown 이전은 `knowledge-transfer`, 새 지식 추출은 `knowledge-scan` 소유. 여러 묶음 미리 보기: 같은 바이트 자동 중복 정리, 입력 지문·의미 후보·같은 경로 수정본 반환. 활성 host 검토의 `separate`·`equivalent`·`choose` 결정은 한 번의 정본 적용에 연결. 통합·미선택 Wiki 원본은 활성 검색 밖의 이식 가능한 merge provenance 보존. 비공개 모음은 승인 연결 전 분리 보관. FTS와 선택형 벡터 결정은 정본 이전과 분리.
