---
schema_version: 1
pair_id: source-knowledge-scan-validation
topic_slug: source-knowledge-scan-validation
language: ko
counterpart: ../en/source-knowledge-scan-validation.md
title: "검토 지식 스캔 검증 정합성"
summary: "candidate·apply 공통 credential 검증과 사람용 review ID 오인 방지"
tags: [knowledge, scan, source, v0-9-4, validation]
aliases: ["검토 source 가져오기", "스캔 검증 정합성"]
sources:
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:0ffab09ca1eaac47b41608e04003fdbc51ee7e5edbaf2386554c885d9f55ec58"
  - "repo:crates/hive-wiki/src/store.rs#sha256:8cc43d6115ca841cdb540c9cabc0a943b5931f760728481f1a97767005902e84"
links: [knowledge-cross-project-access, knowledge-portability-scan, source-development]
reviewed_revision: "git:1cdebf8aef9dde963f5aa88a36042186fbf37e80"
status: active
---

# 검토 지식 스캔 검증 정합성

`hive knowledge scan --candidates`와 `--apply`의 검토 claim credential 검증 공통화. registry·index
mutation 전 거부, 오류에는 raw source 대신 reviewed claim ID와 statement field 표시.

canonical scan provenance의 사람용 요약에서 review ID 제거, typed metadata 유지. 일반 설명형 ID의
opaque credential 오인 방지. source claim: project-private 유지, explicit collection 조회 필요.
