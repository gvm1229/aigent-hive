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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:db6b864c91ed2e96f16481fe9bf8818ebb6955de1760e41f753a424a3c310ea5"
  - "repo:crates/hive-cli/src/knowledge_transfer.rs#sha256:d1a6df6babfbed54b46bb505889921a30fe86fd14fbd4cc0230d51bf7a99de92"
  - "repo:crates/hive-wiki/src/bundle_store.rs#sha256:5dae98a4757bee2d7025ac3ec711e31ae4a33200b04d9b5aaad8746267a97e89"
  - "repo:docs/guides/knowledge-transfer.md#sha256:f0f48767f5fcb83450853ddf95d0c9f3f573b3c2fb3e0b626adb5af5ed742bf1"
  - "repo:harness/skills/knowledge-manage/references/transfer.md#sha256:67ae49affca4cbd70567fc92eb0f06f105be2ad1d728b1bfad5c783dfc82835f"
links: [global-knowledge-bundle-transfer, knowledge-storage]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# 컴퓨터 간 지식 이전 흐름

기존 Markdown 이전은 `knowledge-transfer`, 새 지식 추출은 `knowledge-scan` 소유. 여러 묶음 미리 보기: 같은 바이트 자동 중복 정리, 입력 지문·의미 후보·같은 경로 수정본 반환. 활성 host 검토의 `separate`·`equivalent`·`choose` 결정은 한 번의 정본 적용에 연결. 통합·미선택 Wiki 원본은 활성 검색 밖의 이식 가능한 merge provenance 보존. 비공개 모음은 승인 연결 전 분리 보관. FTS와 선택형 벡터 결정은 정본 이전과 분리.
