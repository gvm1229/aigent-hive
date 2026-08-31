---
schema_version: 1
pair_id: knowledge-transfer-workflow
topic_slug: knowledge-transfer-workflow
language: ko
counterpart: ../en/knowledge-transfer-workflow.md
title: "컴퓨터 간 지식 이전 흐름"
summary: "묶음·대상 원문 지문에 연결된 가져오기와 사용자 설정을 보존하는 벡터 지연 선택"
tags: [knowledge, portability]
aliases: []
sources:
  - "repo:crates/hive-cli/src/knowledge_transfer.rs#sha256:d1a6df6babfbed54b46bb505889921a30fe86fd14fbd4cc0230d51bf7a99de92"
  - "repo:crates/hive-wiki/src/bundle_store.rs#sha256:46f4d198668dc35e687d07331e7eaaed304d6d99d23582e85ba110331141ed34"
  - "repo:docs/guides/knowledge-transfer.md#sha256:20f057772eec3f009864ab41e104bb265ad5171f3344d826cb2892551a0882b9"
links: [global-knowledge-bundle-transfer, knowledge-storage]
reviewed_revision: "git:523892f0009d7ee04af9381981cb41ba01c4045d"
status: active
---

# 컴퓨터 간 지식 이전 흐름

기존 Markdown 이전은 `knowledge-transfer`, 새 지식 추출은 `knowledge-scan` 소유. 적용 시 전달받은 SHA-256과 검토한 대상 원문 지문을 쓰기 잠금 안에서 대조. 충돌 제외는 명시 선택, 기존 파일 보존. 비공개 모음은 승인 연결 전 분리 보관. FTS 준비와 벡터 작업의 완료 상태 분리. 이전 기록에 예·아니요·취소 의미 보존, 전역 벡터 사용 설정 변경 없음. 재생성 범위는 가져온 모음의 기존 승인 범위만 포함. 미설치·미승인 상태의 자동 설치 금지.
